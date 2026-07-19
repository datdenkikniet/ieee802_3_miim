//! State machine for determining a [`Miim`]'s link state.
//!
//! Determine a PHY's [`LinkState`] based on the contents of the base set
//! of MIIM registers. This code requires _no_ PHY-specific code beyond
//! what is provided by [`Miim::read_raw`], and is the logic backing
//! [`Miim::get_link_state`].
///
// All relevant bits (ignoring 100BASE-T2 and 100BASE-T4) are in IEEE 802.3-2022:
// * 22.2.4.1 Control Register (Register 0)
// * 22.2.4.2 Status register (Register 1)
// * 22.2.4.3.7 MASTER-SLAVE control register (Register 9)
//   which links to 40.5.1.1 1000BASE-T use of registers during Auto-Negotiation
// * 22.2.4.3.8 MASTER-SLAVE status register (Register 10)
//   which links to 40.5.1.1 1000BASE-T use of registers during Auto-Negotiation
// * 28.2.4.1.3 Auto-Negotiation advertisement register (Register 4)
// * 28.2.4.1.4 Auto-Negotiation Link Partner ability register (Register 5)
// * 28.2.4.1.5 Auto-Negotiation expansion register (Register 6) (RO)

#[cfg(doc)]
use crate::Miim;

use crate::{
    registers::{auto_negotiation::*, leader_follower::*, *},
    LinkSpeed, LinkState,
};

use core::ops::ControlFlow::{Break, Continue};

type Control<T> = core::ops::ControlFlow<Result<LinkState, LinkStateError>, T>;

/// Errors that can occur when attempting to determine
/// the state of a link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LinkStateError {
    /// No link has been established yet.
    NoLink,
    /// Autonegotiation is enabled, but has not completed yet.
    AutonegotiationNotCompleted,
    /// Autonegotiation is enabled, but the PHY does not support extended
    /// capabilities. This means that it lacks the registers required to
    /// read out the autonegotiation status, so the status cannot be read out.
    ExtendedCapabilities,
    /// Autonegotiation is enabled, but he link partner does not support auto negotiation.
    LinkPartnerNotAutonegotiationAble,
    /// None of the technologies supported by this PHY
    /// are supported by the autonegotitation link partner, and vice-versa.
    NoMatchingTechnologies,
}

// Priority resolution as defined in IEEE 802.3-2022, Section 28B.3
// 100BASE-T2 and 100BASE-T4 are ignored
fn non_gigabit_link_state(
    local_ta: TechnologyAbility,
    lp_ta: TechnologyAbility,
) -> Result<LinkState, LinkStateError> {
    let local_100_fd = local_ta._100base_tx_fd();
    let local_100_hd = local_ta._100base_tx_hd();
    let local_10_fd = local_ta._10base_t_fd();
    let local_10_hd = local_ta._10base_t_fd();

    let lp_100_fd = lp_ta._100base_tx_fd();
    let lp_100_hd = lp_ta._100base_tx_hd();
    let lp_10_fd = lp_ta._10base_t_fd();
    let lp_10_hd = lp_ta._10base_t_hd();

    // Priority resolution as defined in IEEE 802.3-2022, Section 28B.3
    // 100BASE-T2 and 100BASE-T4 are ignored
    let (speed, duplex) = if local_100_fd && lp_100_fd {
        (LinkSpeed::Mbps100, Duplex::Full)
    } else if local_100_hd && lp_100_hd {
        (LinkSpeed::Mbps100, Duplex::Half)
    } else if local_10_fd && lp_10_fd {
        (LinkSpeed::Mbps10, Duplex::Full)
    } else if local_10_hd && lp_10_hd {
        (LinkSpeed::Mbps10, Duplex::Half)
    } else {
        return Result::Err(LinkStateError::NoMatchingTechnologies);
    };

    Result::Ok(LinkState { speed, duplex })
}

/// A state machine describing the process of determining the link state of
/// an (R)(G)MII PHY.
///
/// Each step in the process returns a [`ControlFlow`][`core::ops::ControlFlow`]. The `Break`
/// variant indicates that a result has been obtained, while the [`Continue`] variant indicates
/// that additional register reads are required. The value in the [`Continue`] variant will
/// have a `next` function to drive the process further.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GetLinkStateProcess;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Autonegotiating {
    pub status: BasicStatus,
    pub control: BasicControl,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AutonegotiatingGigabit {
    pub advertisement: AutonegotiationAdvertisement,
    pub link_partner_ability: AutonegotiationLinkPartnerAbility,
}

impl GetLinkStateProcess {
    /// Perform the process in a single shot by providing all of the registers
    /// necessary to perform the determination.
    pub fn oneshot(
        status: BasicStatus,
        control: BasicControl,
        autoneg_exp: AutonegotiationExpansion,
        advertisement: AutonegotiationAdvertisement,
        link_partner_ability: AutonegotiationLinkPartnerAbility,
        lf_control: LeaderFollowerControl,
        lf_status: LeaderFollowerStatus,
    ) -> Result<LinkState, LinkStateError> {
        let process = match GetLinkStateProcess::start(status, control) {
            Continue(c) => c,
            Break(res) => return res,
        };

        let process = match process.next(autoneg_exp, advertisement, link_partner_ability) {
            Continue(c) => c,
            Break(res) => return res,
        };

        process.next(lf_control, lf_status)
    }

    /// Start the process of determining the link state of a PHY.
    pub fn start(status: BasicStatus, control: BasicControl) -> Control<Autonegotiating> {
        if !status.link_status() {
            return Break(Err(LinkStateError::NoLink));
        }

        let link_config = control.get_link_config();
        let autoneg_completed = status.autonegotiation_complete();

        let result = match (link_config, autoneg_completed) {
            (LinkConfig::Manual { duplex, speed }, _) => Ok(LinkState {
                speed,
                duplex: match duplex {
                    DuplexConfig::Half => Duplex::Half,
                    DuplexConfig::Full { .. } => Duplex::Full,
                },
            }),
            (LinkConfig::Autonegotiate { .. }, false) => {
                Err(LinkStateError::AutonegotiationNotCompleted)
            }
            (LinkConfig::Autonegotiate { .. }, true) => {
                if !status.extended_capabilities() {
                    return Break(Err(LinkStateError::ExtendedCapabilities));
                }

                return Continue(Autonegotiating { status, control });
            }
        };

        Break(result)
    }
}

impl Autonegotiating {
    /// Perform the next step of determining the link state.
    pub fn next(
        self,
        autoneg_exp: AutonegotiationExpansion,
        advertisement: AutonegotiationAdvertisement,
        link_partner_ability: AutonegotiationLinkPartnerAbility,
    ) -> Control<AutonegotiatingGigabit> {
        if !autoneg_exp.link_partner_autonegotiation_able() {
            return Break(Err(LinkStateError::LinkPartnerNotAutonegotiationAble));
        }

        // Having extended status is equivalent to being 1000 Mbit capable
        let gigabit = self.status.extended_status();

        // According to 802.3-2022, Table 40-3, the LEADER-FOLLOWER status bits
        // are only valid if 6.1 Page Received bit has been set.
        //
        // However, this bit latches low, which means we can only use it to
        // read the correct link state exactly once. Instead, we will assume
        // that the link partner being next page able is enough of an indication
        // of gigabit-ability.
        let partner_np_able = autoneg_exp.link_partner_next_page_able();

        if gigabit && partner_np_able {
            return Continue(AutonegotiatingGigabit {
                advertisement,
                link_partner_ability,
            });
        }

        Break(non_gigabit_link_state(
            advertisement.technology_ability(),
            link_partner_ability.technology_ability(),
        ))
    }
}

impl AutonegotiatingGigabit {
    /// Performt the next step in the autonegotiation process. In the case of this step,
    /// a [`Result`] is guaranteed to be returned.
    pub fn next(
        self,
        lf_control: LeaderFollowerControl,
        lf_status: LeaderFollowerStatus,
    ) -> Result<LinkState, LinkStateError> {
        let local_1000_fd = lf_control._1000base_t_fd();
        let local_1000_hd = lf_control._1000base_t_hd();

        let lp_1000_fd = lf_status._1000base_t_fd();
        let lp_1000_hd = lf_status._1000base_t_hd();

        if local_1000_fd && lp_1000_fd {
            Ok(LinkState {
                speed: LinkSpeed::Mbps1000,
                duplex: Duplex::Full,
            })
        } else if local_1000_hd && lp_1000_hd {
            Ok(LinkState {
                speed: LinkSpeed::Mbps1000,
                duplex: Duplex::Half,
            })
        } else {
            non_gigabit_link_state(
                self.advertisement.technology_ability(),
                self.link_partner_ability.technology_ability(),
            )
        }
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;
    use std::ops::ControlFlow;

    use crate::{
        registers::{
            auto_negotiation::AutonegotiationExpansion,
            leader_follower::{LeaderFollowerControl, LeaderFollowerStatus},
            BasicControl, BasicStatus, Duplex, DuplexConfig, LinkConfig,
        },
        GetLinkStateProcess, LinkSpeed, LinkState, LinkStateError,
    };

    fn control_to_determined_pairs() -> impl Iterator<Item = (LinkConfig, LinkState)> {
        [LinkSpeed::Mbps10, LinkSpeed::Mbps100, LinkSpeed::Mbps1000]
            .into_iter()
            .flat_map(|speed| {
                [
                    (
                        LinkConfig::Manual {
                            duplex: DuplexConfig::Half,
                            speed,
                        },
                        LinkState {
                            speed,
                            duplex: Duplex::Half,
                        },
                    ),
                    (
                        LinkConfig::Manual {
                            duplex: DuplexConfig::Full {
                                unidirectional: true,
                            },
                            speed,
                        },
                        LinkState {
                            speed,
                            duplex: Duplex::Full,
                        },
                    ),
                    (
                        LinkConfig::Manual {
                            duplex: DuplexConfig::Full {
                                unidirectional: false,
                            },
                            speed,
                        },
                        LinkState {
                            speed,
                            duplex: Duplex::Full,
                        },
                    ),
                ]
            })
    }

    #[test]
    fn no_link() {
        let mut status = BasicStatus::from(0);
        status.set_link_status(false);

        let control = BasicControl::from(0);
        let start = GetLinkStateProcess::start(status, control);

        assert_matches!(start, ControlFlow::Break(Err(LinkStateError::NoLink)));
    }

    #[test]
    fn link_no_autonegotiation() {
        for (config, expected) in control_to_determined_pairs() {
            let mut status = BasicStatus::from(0);
            status.set_link_status(true);
            status.set_autonegotiation_complete(false);

            let mut control = BasicControl::from(0);
            control.set_link_config(config);

            let start = GetLinkStateProcess::start(status, control);

            assert_matches!(start, ControlFlow::Break(Ok(s)) if s == expected, "{:?} != {:?}", start, expected);
        }
    }

    #[test]
    fn link_autoneg_incomplete() {
        let mut status = BasicStatus::from(0);
        status.set_link_status(true);
        status.set_autonegotiation_complete(false);
        status.set_extended_capabilities(true);

        let mut control = BasicControl::from(0);
        control.set_link_config(LinkConfig::Autonegotiate { restart: false });

        let start = GetLinkStateProcess::start(status, control);

        assert_matches!(
            start,
            ControlFlow::Break(Err(LinkStateError::AutonegotiationNotCompleted))
        );
    }

    #[test]
    fn link_autonegotiation_no_ext_cap() {
        let mut status = BasicStatus::from(0);
        status.set_link_status(true);
        status.set_autonegotiation_complete(true);

        let mut control = BasicControl::from(0);
        control.set_link_config(LinkConfig::Autonegotiate { restart: false });

        let start = GetLinkStateProcess::start(status, control);

        assert_matches!(
            start,
            ControlFlow::Break(Err(LinkStateError::ExtendedCapabilities))
        );
    }

    #[test]
    fn link_autonegotiation_complete() {
        let mut status = BasicStatus::from(0);
        status.set_link_status(true);
        status.set_autonegotiation_complete(true);
        status.set_extended_capabilities(true);

        let mut control = BasicControl::from(0);
        control.set_link_config(LinkConfig::Autonegotiate { restart: false });

        let start = GetLinkStateProcess::start(status, control);

        assert_matches!(start, ControlFlow::Continue(_));
    }

    #[test]
    fn link_autonegotiation_partner_unable() {
        let mut status = BasicStatus::from(0);
        status.set_link_status(true);
        status.set_autonegotiation_complete(true);
        status.set_extended_capabilities(true);

        let mut control = BasicControl::from(0);
        control.set_link_config(LinkConfig::Autonegotiate { restart: false });

        let start = GetLinkStateProcess::start(status, control);
        let autoneg = start.continue_value().unwrap();

        let autoneg = autoneg.next(0u16.into(), 0u16.into(), 0u16.into());

        assert_matches!(
            autoneg,
            ControlFlow::Break(Err(LinkStateError::LinkPartnerNotAutonegotiationAble))
        );
    }

    #[test]
    fn link_autonegotiation_non_gmii_to_non_gmii() {
        // NB: all non-`true, true` combinations represent a non-gmii
        // (advertising) PHY autonegotiating with another non-gmii (advertising) PHY.
        for (ext_status, remote_next_page) in [(false, false), (false, true), (false, true)] {
            let mut status = BasicStatus::from(0);
            status.set_link_status(true);
            status.set_autonegotiation_complete(true);
            status.set_extended_capabilities(true);
            status.set_extended_status(ext_status);

            let mut control = BasicControl::from(0);
            control.set_link_config(LinkConfig::Autonegotiate { restart: false });

            let start = GetLinkStateProcess::start(status, control);
            let autoneg = start.continue_ok().unwrap();

            let mut exp = AutonegotiationExpansion::from(0);
            exp.set_link_partner_autonegotiation_able(true);
            exp.set_link_partner_next_page_able(remote_next_page);

            let autoneg = autoneg.next(exp, 0u16.into(), 0u16.into());

            // NOTE: only `non_gigabit_link_state` can return this error variant.
            assert_matches!(
                autoneg,
                ControlFlow::Break(Err(LinkStateError::NoMatchingTechnologies)),
                "ext_status: {ext_status}, remote_next_page: {remote_next_page}",
            );
        }
    }

    #[test]
    fn link_autonegotiation_gmii_to_gmii() {
        #[rustfmt::skip]
        const COMBINATIONS: [(bool, bool); 4] = [
            (true,  true),
            (false, true),
            (true,  false),
            (false, false)
        ];

        let cases = COMBINATIONS.into_iter().flat_map(|hd| {
            COMBINATIONS.into_iter().map(move |fd| {
                let duplex = if fd.0 && fd.1 {
                    Some(Duplex::Full)
                } else if hd.0 && hd.1 {
                    Some(Duplex::Half)
                } else {
                    None
                };

                (hd, fd, duplex)
            })
        });

        for tuple in cases {
            let ((local_hd, remote_hd), (local_fd, remote_fd), expected) = tuple;
            let mut status = BasicStatus::from(0);
            status.set_link_status(true);
            status.set_autonegotiation_complete(true);
            status.set_extended_capabilities(true);
            status.set_extended_status(true);

            let mut control = BasicControl::from(0);
            control.set_link_config(LinkConfig::Autonegotiate { restart: false });

            let start = GetLinkStateProcess::start(status, control);
            let autoneg = start.continue_value().unwrap();

            let mut exp = AutonegotiationExpansion::from(0);
            exp.set_link_partner_autonegotiation_able(true);
            exp.set_next_page_able(true);
            exp.set_link_partner_next_page_able(true);

            let autoneg_gbit = autoneg
                .next(exp, 0u16.into(), 0u16.into())
                .continue_ok()
                .unwrap();

            let mut lf_control = LeaderFollowerControl::from(0);
            lf_control.set__1000base_t_fd(local_fd);
            lf_control.set__1000base_t_hd(local_hd);

            let mut lf_status = LeaderFollowerStatus::from(0);
            lf_status.set__1000base_t_fd(remote_fd);
            lf_status.set__1000base_t_hd(remote_hd);

            let gbit = autoneg_gbit.next(lf_control, lf_status);

            if let Some(expected) = expected {
                assert_matches!(
                    gbit,
                    Ok(
                        LinkState {
                        speed: LinkSpeed::Mbps1000,
                        duplex
                    }) if duplex == expected,
                    "{tuple:?}",
                )
            } else {
                // NOTE: only `non_gigabit_link_state` can return this error variant.
                assert_matches!(gbit, Err(LinkStateError::NoMatchingTechnologies));
            }
        }
    }
}
