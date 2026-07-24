//! The basic register set.

use crate::{
    registers::{Register, RegisterAddress},
    LinkSpeed,
};

/// The Status Register
///
/// This register reports part of the PHY's capabilities, and contains status
/// flags.
///
// Defined in 22.2.4.2.
#[bitbybit::bitfield(u16, forbid_overlaps, default = 0, defmt_bitfields(feature = "defmt"))]
#[derive(Debug, PartialEq)]
pub struct BasicStatus {
    /// Whether this PHY supports the extended capability registers (2-14, 16-31) defined
    /// by the standard.
    #[bit(0, rw)]
    pub extended_capabilities: bool,
    /// Whether this PHY has detected a jabber event.
    ///
    /// This bit is cleared on a read of the register.
    #[bit(1, rw)]
    pub jabber_detect: bool,
    /// Whether the PHY has detected a valid link.
    #[bit(2, rw)]
    pub link_status: bool,
    /// Whether this PHY is able to perform autonegotiation.
    #[bit(3, rw)]
    pub autonegotiate_able: bool,
    /// A remote fault was detected.
    #[bit(4, rw)]
    pub remote_fault: bool,
    /// Autonegotiation has completed.
    ///
    /// Once this bit is set, the auto negotiation registers for
    /// the PHY are valid.
    #[bit(5, rw)]
    pub autonegotiation_complete: bool,
    /// Whether this PHY supports reading management frames without
    /// management preamble.
    ///
    /// See 22.2.4.6.2 for more information about what this bit
    /// can be used for.
    #[bit(6, rw)]
    pub mf_preamble_suppression: bool,
    /// Whether this PHY supports transmitting regardless of whether
    /// it has established a valid link.
    #[bit(7, rw)]
    pub unidirectional_ability: bool,
    /// The PHY supports the [`ExtendedStatus`] register.
    #[bit(8, rw)]
    pub extended_status: bool,
    /// The PHY supports 100BASE-T2, Half-Duplex.
    #[bit(9, rw)]
    pub _100base_t2_hd: bool,
    /// The PHY supports 100BASE-T2, Full-Duplex.
    #[bit(10, rw)]
    pub _100base_t2_fd: bool,
    /// The PHY supports 10BASE-T, Half-Duplex.
    #[bit(11, rw)]
    pub _10base_t_hd: bool,
    /// The PHY supports 10BASE-T, Full-Duplex.
    #[bit(12, rw)]
    pub _10base_t_fd: bool,
    /// The PHY supports 100BASE-X, Half-Duplex.
    #[bit(13, rw)]
    pub _100base_x_hd: bool,
    /// The PHY supports 100BASE-X, Full-Duplex.
    #[bit(14, rw)]
    pub _100base_x_fd: bool,
    /// The PHY supports 100BASE-T4.
    #[bit(15, rw)]
    pub _100base_t4: bool,
}

from_into!(BasicStatus);

impl Register for BasicStatus {
    const ADDRESS: RegisterAddress = RegisterAddress::new(1).unwrap();
}

/// The extended status register, containing additional status bits.
///
/// This register is RESERVED on MII PHYs, but is part of the base
/// set on GMII PHYs (i.e. standard-conforming MII PHYs _must not_ support
/// this register, and standard-conforming GMII PHYs _must_ support this
/// register).
///
// Defined in 22.2.4.4.
#[bitbybit::bitfield(u16, forbid_overlaps, default = 0, defmt_bitfields(feature = "defmt"))]
#[derive(Debug, PartialEq)]
pub struct ExtendedStatus {
    #[bit(12, rw)]
    /// The PHY spuports 1000BASE-T, Half-Duplex.
    pub _1000base_t_hd: bool,
    /// The PHY spuports 1000BASE-T, Full-Duplex.
    #[bit(13, rw)]
    pub _1000base_t_fd: bool,
    /// The PHY spuports 1000BASE-X, Half-Duplex.
    #[bit(14, rw)]
    pub _1000base_x_hd: bool,
    /// The PHY spuports 1000BASE-X, Full-Duplex.
    #[bit(15, rw)]
    pub _1000base_x_fd: bool,
}

from_into!(ExtendedStatus);

impl Register for ExtendedStatus {
    const ADDRESS: RegisterAddress = RegisterAddress::new(15).unwrap();
}

/// A duplex mode.
#[bitbybit::bitenum(u1, exhaustive = true)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Duplex {
    /// Full duplex.
    Full = 1,
    /// Half duplex.
    Half = 0,
}

/// Valid duplex mode configurations for [`BasicControl`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DuplexConfig {
    /// Half duplex.
    Half,
    /// Full duplex.
    Full {
        /// If `true`, the PHY will transmit regardless of the link
        /// status. If `false`, the PHY will only transmit if it deems
        /// that a valid link has been established.
        unidirectional: bool,
    },
}

/// Link mode configurations available in for [`BasicControl`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LinkConfig {
    /// Use autonegotiation to configure the link state.
    Autonegotiate {
        /// Restart autonegotiation.
        restart: bool,
    },
    /// Manual configuration of the link state.
    Manual {
        /// The duplex to use.
        duplex: DuplexConfig,
        /// The speed to use.
        speed: LinkSpeed,
    },
}

impl LinkConfig {
    /// Check if this link configuration uses autonegotiation.
    pub fn is_autonegotiation(&self) -> bool {
        matches!(self, Self::Autonegotiate { restart: _ })
    }
}

/// The Control Register, containing basic control bits.
///
// Defined in 37.2.5.1.1.
#[bitbybit::bitfield(u16, forbid_overlaps, default = 0, defmt_bitfields(feature = "defmt"))]
#[derive(Debug, PartialEq)]
pub struct BasicControl {
    /// Do not access this field directly: use [`BasicControl::set_link_config`]
    /// and [`BasicControl::get_link_config`] instead.
    #[bit(5, rw)]
    unidirectional_enable: bool,
    /// Do not access this field directly: use [`BasicControl::set_link_config`]
    /// and [`BasicControl::get_link_config`] instead.
    #[bit(6, rw)]
    speed_sel_msb: bool,
    /// Enable the collision test signal.
    #[bit(7, rw)]
    pub collision_test: bool,
    /// Do not access this field directly: use [`BasicControl::set_link_config`]
    /// and [`BasicControl::get_link_config`] instead.
    #[bit(8, rw)]
    duplex_mode: Duplex,
    /// Do not access this field directly: use [`BasicControl::set_link_config`]
    /// and [`BasicControl::get_link_config`] instead.
    #[bit(9, rw)]
    restart_autonegotiation: bool,
    /// Electrically isolate the PHY from its (Gigabit) Media Independent
    /// Interface.
    ///
    /// In other words, disconnect the PHY from the pins it shares
    /// with the MAC component connected to it. Setting this bit
    /// _does_ retain access to the PHY over MIIM.
    #[bit(10, rw)]
    pub isolate: bool,
    /// Enable Power-Down mode.
    #[bit(11, rw)]
    pub power_down: bool,
    /// Do not access this field directly: use [`BasicControl::set_link_config`]
    /// and [`BasicControl::get_link_config`] instead.
    #[bit(12, rw)]
    autonegotiation_enable: bool,
    /// Do not access this field directly: use [`BasicControl::set_link_config`]
    /// and [`BasicControl::get_link_config`] instead.
    #[bit(13, rw)]
    speed_sel_lsb: bool,
    /// Enable loopback mode.
    #[bit(14, rw)]
    pub loopback: bool,
    /// Perform a reset.
    ///
    /// If set, this bit will stay high until the reset
    /// has completed.
    #[bit(15, rw)]
    pub reset: bool,
}

impl BasicControl {
    /// Get the current link configuration.
    pub fn get_link_config(&self) -> LinkConfig {
        if self.autonegotiation_enable() {
            LinkConfig::Autonegotiate {
                restart: self.restart_autonegotiation(),
            }
        } else {
            let duplex = match self.duplex_mode() {
                Duplex::Half => DuplexConfig::Half,
                Duplex::Full => DuplexConfig::Full {
                    unidirectional: self.unidirectional_enable(),
                },
            };

            let speed = match (self.speed_sel_lsb(), self.speed_sel_msb()) {
                (false, true) => LinkSpeed::Mbps1000,
                (true, false) => LinkSpeed::Mbps100,
                (false, false) => LinkSpeed::Mbps10,
                (true, true) => panic!("PHY reported invalid speed 0b11"),
            };

            LinkConfig::Manual { duplex, speed }
        }
    }

    /// Set the link configuration to `config`.
    pub fn set_link_config(&mut self, config: LinkConfig) {
        match config {
            LinkConfig::Autonegotiate { restart } => {
                self.set_autonegotiation_enable(true);
                self.set_restart_autonegotiation(restart);
            }
            LinkConfig::Manual { duplex, speed } => {
                self.set_autonegotiation_enable(false);
                match duplex {
                    DuplexConfig::Half => self.set_duplex_mode(Duplex::Half),
                    DuplexConfig::Full { unidirectional } => {
                        self.set_duplex_mode(Duplex::Full);
                        self.set_unidirectional_enable(unidirectional);
                    }
                }

                let (lsb, msb) = match speed {
                    LinkSpeed::Mbps1000 => (false, true),
                    LinkSpeed::Mbps100 => (true, false),
                    LinkSpeed::Mbps10 => (false, false),
                };

                self.set_speed_sel_lsb(lsb);
                self.set_speed_sel_msb(msb);
            }
        }
    }
}

from_into!(BasicControl);

impl Register for BasicControl {
    const ADDRESS: RegisterAddress = RegisterAddress::new(0).unwrap();
}

#[cfg(test)]
mod test {
    use crate::{
        registers::{BasicControl, Duplex},
        LinkSpeed,
    };

    use super::{DuplexConfig, LinkConfig};

    #[test]
    fn set_manual_mode_disables_autonegotiation() {
        let mut status = BasicControl::from(0);
        status.set_link_config(LinkConfig::Manual {
            duplex: DuplexConfig::Half,
            speed: LinkSpeed::Mbps100,
        });

        assert!(!status.autonegotiation_enable());
        assert!(status.speed_sel_lsb());
        assert!(!status.speed_sel_msb());
        assert_eq!(status.duplex_mode(), Duplex::Half);
    }
}
