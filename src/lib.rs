#![cfg_attr(not(test), no_std)]
#![deny(missing_docs)]

//! A crate that provides access to the MIIM interface described
//! by IEEE standard 802.3

pub mod mdio;

#[cfg(feature = "mmd")]
mod mmd;
#[cfg(feature = "mmd")]
use mmd::Mmd;

pub mod registers;
use registers::*;

use crate::registers::{
    auto_negotiation::{
        AutonegotiationAdvertisement, AutonegotiationExpansion, AutonegotiationLinkPartnerAbility,
    },
    leader_follower::{LeaderFollowerControl, LeaderFollowerStatus},
};

#[cfg(feature = "phy")]
pub mod phy;

/// Errors that can occur when attempting to determine
/// the state of a link.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkStateError {
    /// No link has been established yet.
    NoLink,
    /// The PHY is configured for autonegotiation, but it does
    /// not support autonegotiation.
    NotAutonegotiationAble,
    /// Autonegotiation is enabled, but has not completed yet.
    AutonegotiationNotCompleted,
    /// An autonegotiating PHY without extended capabilities
    /// was encountered. Reading out the link state for such
    /// a PHY is not possible due to missing register.
    ExtendedCapabilities,
    /// The link partner does not support auto negotiation.
    LinkPartnerNotAutonegotiationAble,
    /// None of the technologies supported by this PHY
    /// are supported by the link partner, and vice-versa.
    NoMatchingTechnologies,
}

/// The state of a link.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkState {
    /// The speed of the link.
    pub speed: LinkSpeed,
    /// The duplex mode of the link.
    pub duplex: DuplexMode,
}

/// All basic link speeds possibly supported by the PHY.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkSpeed {
    /// 1000 Mbps
    Mbps1000,
    /// 100 Mbps
    Mbps100,
    /// 10 Mbps
    Mbps10,
}

/// The PHY IDENT of this PHY
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhyIdent(u16, u16);

impl PhyIdent {
    /// Create a new PhyIdent
    pub fn new(phy_ident_1: u16, phy_ident_2: u16) -> Self {
        Self(phy_ident_1, phy_ident_2)
    }

    /// The raw values of this PhyIdent
    pub fn raw(&self) -> (u16, u16) {
        (self.0, self.1)
    }

    /// The raw value of this PhyIdent, as u32
    pub fn raw_u32(&self) -> u32 {
        (self.0 as u32) << 16 | (self.1 as u32)
    }

    /// The OUI of this PhyIdent
    pub fn oui(&self) -> u32 {
        (self.0 as u32) << 6 & (self.1 as u32) >> 10
    }

    /// The model number of this PhyIdent
    pub fn model_number(&self) -> u8 {
        (self.1 >> 4) as u8 & 0x3F
    }

    /// The revision number of this PhyIdent
    pub fn revision(&self) -> u8 {
        (self.1) as u8 & 0x0F
    }
}

/// An IEEE 802.3 compatible PHY that can be managed
/// using the Media Independent Interface Management (MIIM)
/// Interface.
pub trait Miim {
    /// Read the MIIM register at `address`.
    fn read_raw(&mut self, address: u8) -> u16;

    /// Write `value` to the MIIM register at `address`.
    fn write_raw(&mut self, address: u8, value: u16);

    /// Read the register `Register` on this PHY.
    fn read<R: Register>(&mut self) -> R {
        let raw = self.read_raw(R::ADDRESS);
        raw.into()
    }

    /// Write the register `Register`.
    fn write<R: Register>(&mut self, value: R) {
        let raw = value.into();
        self.write_raw(R::ADDRESS, raw);
    }

    /// Modify the register `Register` on this PHY.
    fn modify<R: Register, F>(&mut self, f: F)
    where
        F: FnOnce(&mut R),
    {
        let mut value = self.read();
        f(&mut value);
        self.write(value);
    }

    /// Check if the PHY is currently resetting
    fn is_resetting(&mut self) -> bool {
        self.read::<BasicControl>().reset()
    }

    /// Reset the PHY. Verify that the reset has completed by checking
    /// [`Self::is_resetting`] == false before continuing usage.
    fn reset(&mut self) {
        self.modify(|bcr: &mut BasicControl| {
            bcr.set_reset(true);
        });
    }

    /// Perform a reset, blocking until the reset is completed
    fn blocking_reset(&mut self) {
        self.reset();
        while self.is_resetting() {}
    }

    /// Get the raw value of the Base Status Register of this PHY
    fn status(&mut self) -> BasicStatus {
        self.read()
    }

    /// Check if the PHY reports its link as being up
    fn phy_link_up(&mut self) -> bool {
        self.status().link_status()
    }

    /// Read the PHY identifier for this PHY.
    ///
    /// Returns `None` if `extended_capabilities` in [`Self::status`] is false
    fn phy_ident(&mut self) -> Option<PhyIdent> {
        if self.status().extended_capabilities() {
            let msb = self.read_raw(2);
            let lsb = self.read_raw(3);
            Some(PhyIdent::new(msb, lsb))
        } else {
            None
        }
    }

    /// The best advertisement this PHY supports and restart autonegotation.
    ///
    /// "Best", in this case, means largest amount of supported features
    fn set_best_autonegotation_advertisement(&mut self) {
        let status: BasicStatus = self.read();

        // Extended capabilities are required to configure
        // autonegotiation.
        if !status.extended_capabilities() {
            return;
        }

        // Extended status == 1000BASE-T able
        if status.extended_status() {
            let extended: ExtendedStatus = self.read();
            let _1000base_t_fd = extended._1000base_t_fd();
            let _1000base_t_hd = extended._1000base_t_hd();

            self.modify(|r: &mut LeaderFollowerControl| {
                r.set__1000base_t_fd(_1000base_t_fd);
                r.set__1000base_t_hd(_1000base_t_hd);
            });
        }

        // Ignore 100baset4
        let _100base_x_fd = status._100base_x_fd();
        let _100base_x_hd = status._100base_x_hd();
        let _10base_t_fd = status._10base_t_fd();
        let _10base_t_hd = status._10base_t_hd();

        self.modify(|r: &mut AutonegotiationAdvertisement| {
            let mut tech_ability = r.technology_ability();
            tech_ability.set__100base_tx_fd(_100base_x_fd);
            tech_ability.set__100base_tx_hd(_100base_x_hd);
            tech_ability.set__10base_t_fd(_10base_t_fd);
            tech_ability.set__10base_t_hd(_10base_t_hd);
            r.set_technology_ability(tech_ability);
        });

        self.modify(|bcr: &mut BasicControl| {
            bcr.set_link_config(BasicControlLinkConfig::Autonegotiate { restart: true });
        })
    }

    /// Get the current link speed of this PHY.
    ///
    /// All relevant bits (ignoring 100BASE-T2 and 100BASE-T4) are in IEEE 802.3-2022:
    /// 22.2.4.1 Control Register (Register 0)
    /// 22.2.4.2 Status register (Register 1)
    /// 22.2.4.3.7 MASTER-SLAVE control register (Register 9)
    ///   which links to 40.5.1.1 1000BASE-T use of registers during Auto-Negotiation
    /// 22.2.4.3.8 MASTER-SLAVE status register (Register 10)
    ///   which links to 40.5.1.1 1000BASE-T use of registers during Auto-Negotiation
    /// 28.2.4.1.3 Auto-Negotiation advertisement register (Register 4)
    /// 28.2.4.1.4 Auto-Negotiation Link Partner ability register (Register 5)
    /// 28.2.4.1.5 Auto-Negotiation expansion register (Register 6) (RO)
    fn get_link_state(&mut self) -> Result<LinkState, LinkStateError> {
        let basic_control: BasicControl = self.read();
        let basic_status: BasicStatus = self.read();

        if !basic_status.link_status() {
            return Err(LinkStateError::NoLink);
        }

        // Having extended status is equivalent to being 1000 Mbit capable
        let has_extended_status = basic_status.extended_status();
        let gigabit_able = has_extended_status;

        // We don't need to check if the PHY is autonegotiation able:
        // BasicControl must return 0 for autonegotiation enabled on
        // PHYs that don't support it.
        let link_config = basic_control.get_link_config();
        let autoneg_completed = basic_status.autonegotiation_complete();

        match (link_config, autoneg_completed) {
            (BasicControlLinkConfig::Manual { duplex, speed }, _) => Ok(LinkState {
                speed,
                duplex: match duplex {
                    Duplex::Half => DuplexMode::Half,
                    Duplex::Full { .. } => DuplexMode::Full,
                },
            }),
            (BasicControlLinkConfig::Autonegotiate { .. }, false) => {
                Err(LinkStateError::AutonegotiationNotCompleted)
            }
            (BasicControlLinkConfig::Autonegotiate { .. }, true) => {
                if !basic_status.extended_capabilities() {
                    return Err(LinkStateError::ExtendedCapabilities);
                }

                let autoneg_exp: AutonegotiationExpansion = self.read();
                let advertisement: AutonegotiationAdvertisement = self.read();

                if !autoneg_exp.link_partner_autonegotiation_able() {
                    return Err(LinkStateError::LinkPartnerNotAutonegotiationAble);
                }

                // Priority resolution as defined in IEEE 802.3-2022, Section 28B.3
                // 100BASE-T2 and 100BASE-T4 are ignored
                //
                // IEEE 802.3-2022, Section 40.5.1.2 mandates that 1000BASE-T PHYs
                // must send next pages, so using it as indication for whether
                // the link partner supports gigabit makes sense. Additionally,
                // we know that our local PHY supports it when gigabit is supported,
                // so we can just reuse that knowledge.
                if gigabit_able && autoneg_exp.link_partner_next_page_able() {
                    let lf_control: LeaderFollowerControl = self.read();
                    let lf_status: LeaderFollowerStatus = self.read();

                    // LEADER-FOLLOWER advertisement bits only make sense if we
                    // sent a next page.
                    let local_next_page = autoneg_exp.next_page_able();
                    let local_1000_fd = local_next_page && lf_control._1000base_t_fd();
                    let local_1000_hd = local_next_page && lf_control._1000base_t_hd();

                    // According to 802.3-2022, Table 40-3, the LEADER-FOLLOWER status bits
                    // are only valid if 6.1 Page Received bit has been set.
                    //
                    // However, this bit latches low, which means we can only use it to
                    // read the correct status once. Instead, we will assume
                    // that the link partner being next page able is enough of an indication
                    // of gigabit-ability.
                    let lp_next_page = autoneg_exp.link_partner_next_page_able();
                    let lp_1000_fd = lp_next_page && lf_status._1000base_t_fd();
                    let lp_1000_hd = lp_next_page && lf_status._1000base_t_hd();

                    if local_1000_fd && lp_1000_fd {
                        return Ok(LinkState {
                            speed: LinkSpeed::Mbps1000,
                            duplex: DuplexMode::Full,
                        });
                    } else if local_1000_hd && lp_1000_hd {
                        return Ok(LinkState {
                            speed: LinkSpeed::Mbps1000,
                            duplex: DuplexMode::Half,
                        });
                    }
                }

                let local_ta = advertisement.technology_ability();

                let local_100_fd = local_ta._100base_tx_fd();
                let local_100_hd = local_ta._100base_tx_hd();
                let local_10_fd = local_ta._10base_t_fd();
                let local_10_hd = local_ta._10base_t_fd();

                let link_partner_ability: AutonegotiationLinkPartnerAbility = self.read();
                let lp_ta = link_partner_ability.technology_ability();

                let lp_100_fd = lp_ta._100base_tx_fd();
                let lp_100_hd = lp_ta._100base_tx_hd();
                let lp_10_fd = lp_ta._10base_t_fd();
                let lp_10_hd = lp_ta._10base_t_hd();

                // Priority resolution as defined in IEEE 802.3-2022, Section 28B.3
                // 100BASE-T2 and 100BASE-T4 are ignored
                let (speed, duplex) = if local_100_fd && lp_100_fd {
                    (LinkSpeed::Mbps100, DuplexMode::Full)
                } else if local_100_hd && lp_100_hd {
                    (LinkSpeed::Mbps100, DuplexMode::Half)
                } else if local_10_fd && lp_10_fd {
                    (LinkSpeed::Mbps10, DuplexMode::Full)
                } else if local_10_hd && lp_10_hd {
                    (LinkSpeed::Mbps10, DuplexMode::Half)
                } else {
                    return Err(LinkStateError::NoMatchingTechnologies {});
                };

                Ok(LinkState { speed, duplex })
            }
        }
    }

    /// Set the autonegotiation advertisement and restart the autonegotiation
    /// process
    ///
    /// This is a no-op if `extended_caps` in [`Self::status`] is false
    fn set_autonegotiation_advertisement(&mut self, ad: AutonegotiationAdvertisement) {
        let status = self.status();
        if !status.extended_capabilities() {
            return;
        }

        self.write(ad);

        self.modify(|bcr: &mut BasicControl| {
            bcr.set_link_config(BasicControlLinkConfig::Autonegotiate { restart: true });
        })
    }

    /// Read an MMD register
    #[cfg(feature = "mmd")]
    fn mmd_read(&mut self, mmd_address: u8, reg_address: u16) -> u16
    where
        Self: Sized,
    {
        Mmd::read(self, mmd_address.into(), reg_address)
    }

    /// Write an MMD register
    #[cfg(feature = "mmd")]
    fn mmd_write(&mut self, device_address: u8, reg_address: u16, reg_value: u16)
    where
        Self: Sized,
    {
        Mmd::write(self, device_address.into(), reg_address, reg_value)
    }
}

#[cfg(test)]
mod test {
    use crate::{LinkState, Miim};

    struct MockPhy {
        registers: [u16; 16],
    }

    impl Miim for MockPhy {
        fn read_raw(&mut self, address: u8) -> u16 {
            self.registers[address as usize]
        }

        fn write_raw(&mut self, address: u8, value: u16) {
            self.registers[address as usize] = value;
        }
    }

    const GIGABIT_GIGABIT_PARTNER: MockPhy = MockPhy {
        #[rustfmt::skip]
        registers: [
            0x1000, 0x79ad, 0x001c, 0xc800, 0x0de1, 0xc1e1, 0x006d, 0x2001,
            0x6001, 0x0200, 0x3800, 0x0000, 0x0000, 0x0000, 0x0000, 0x2000,
        ],
    };

    const GIGABIT_100M_PARTNER: MockPhy = MockPhy {
        #[rustfmt::skip]
        registers: [
            0x1040, 0x79ad, 0x001c, 0xc800, 0x0de1, 0x51e1, 0x0065, 0x2001,
            0x0000, 0x0200, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x2000,
        ],
    };

    const GIGABIT_10M_PARTNER: MockPhy = MockPhy {
        #[rustfmt::skip]
        registers: [
        0x1000, 0x79ad, 0x001c, 0xc800, 0x01e1, 0x4061, 0x0067, 0x2801,
        0x0000, 0x0200, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x2000,
        ],
    };

    const GIGABIT_AUTONEG_INCOMPLETE: MockPhy = MockPhy {
        #[rustfmt::skip]
        registers: [
            0x1000, 0x7989, 0x001c, 0xc800, 0x0de1, 0x0000, 0x0064, 2801,
            0x0000, 0x0200, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 2000,
        ],
    };

    #[test]
    fn autoneg_incomplete() {
        let mut phy = GIGABIT_AUTONEG_INCOMPLETE;

        let state = phy.get_link_state();

        assert_eq!(
            state,
            Err(crate::LinkStateError::AutonegotiationNotCompleted)
        )
    }

    #[test]
    fn link_state_10fd() {
        let mut phy = GIGABIT_10M_PARTNER;

        let state = phy.get_link_state().unwrap();

        assert_eq!(
            state,
            LinkState {
                speed: crate::LinkSpeed::Mbps10,
                duplex: crate::registers::DuplexMode::Full
            }
        )
    }

    #[test]
    fn link_state_100fd() {
        let mut phy = GIGABIT_100M_PARTNER;

        let state = phy.get_link_state().unwrap();

        assert_eq!(
            state,
            LinkState {
                speed: crate::LinkSpeed::Mbps100,
                duplex: crate::registers::DuplexMode::Full
            }
        )
    }

    #[test]
    fn link_state_1gfd() {
        let mut phy = GIGABIT_GIGABIT_PARTNER;

        let state = phy.get_link_state().unwrap();

        assert_eq!(
            state,
            LinkState {
                speed: crate::LinkSpeed::Mbps1000,
                duplex: crate::registers::DuplexMode::Full
            }
        )
    }
}
