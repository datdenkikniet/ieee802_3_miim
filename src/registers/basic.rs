//! The basic register set.

use bilge::{bitsize, prelude::*, FromBits};

use crate::{
    registers::{Register, RegisterAddress},
    LinkSpeed,
};

/// The Status Register
///
/// This register reports part of the PHY's capabilities, and contains status
/// flags.
///
/// Defined in 22.2.4.2.
#[bitsize(16)]
#[derive(Clone, Copy, DebugBits, FromBits)]
#[cfg_attr(feature = "defmt", derive(bilge_defmt::FormatBits))]
pub struct BasicStatus {
    /// Whether this PHY supports the extended capability registers (2-14, 16-31) defined
    /// by the standard.
    pub extended_capabilities: bool,
    /// Whether this PHY has detected a jabber event.
    ///
    /// This bit is cleared on a read of the register.
    pub jabber_detect: bool,
    /// Whether the PHY has detected a valid link.
    pub link_status: bool,
    /// Whether this PHY is able to perform autonegotiation.
    pub autonegotiate_able: bool,
    /// A remote fault was detected.
    pub remote_fault: bool,
    /// Autonegotiation has completed.
    ///
    /// Once this bit is set, the auto negotiation registers for
    /// the PHY are valid.
    pub autonegotiation_complete: bool,
    /// Whether this PHY supports reading management frames without
    /// management preamble.
    ///
    /// See 22.2.4.6.2 for more information about what this bit
    /// can be used for.
    pub mf_preamble_suppression: bool,
    /// Whether this PHY supports transmitting regardless of whether
    /// it has established a valid link.
    pub unidirectional_ability: bool,
    /// The PHY supports the [`ExtendedStatus`] register.
    pub extended_status: bool,
    /// The PHY supports 100BASE-T2, Half-Duplex.
    pub _100base_t2_hd: bool,
    /// The PHY supports 100BASE-T2, Full-Duplex.
    pub _100base_t2_fd: bool,
    /// The PHY supports 10BASE-T, Half-Duplex.
    pub _10base_t_hd: bool,
    /// The PHY supports 10BASE-T, Full-Duplex.
    pub _10base_t_fd: bool,
    /// The PHY supports 100BASE-X, Half-Duplex.
    pub _100base_x_hd: bool,
    /// The PHY supports 100BASE-X, Full-Duplex.
    pub _100base_x_fd: bool,
    /// The PHY supports 100BASE-T4.
    pub _100base_t4: bool,
}

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
/// Defined in 22.2.4.4.
#[bitsize(16)]
#[derive(FromBits, DebugBits, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(bilge_defmt::FormatBits))]
pub struct ExtendedStatus {
    reserved: u12,
    /// The PHY spuports 1000BASE-T, Half-Duplex.
    pub _1000base_t_hd: bool,
    /// The PHY spuports 1000BASE-T, Full-Duplex.
    pub _1000base_t_fd: bool,
    /// The PHY spuports 1000BASE-X, Half-Duplex.
    pub _1000base_x_hd: bool,
    /// The PHY spuports 1000BASE-X, Full-Duplex.
    pub _1000base_x_fd: bool,
}

impl Register for ExtendedStatus {
    const ADDRESS: RegisterAddress = RegisterAddress::new(15).unwrap();
}

/// A duplex mode.
#[bitsize(1)]
#[derive(Clone, Copy, Debug, FromBits, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DuplexMode {
    /// Half duplex.
    Half = 0,
    /// Full duplex.
    Full = 1,
}

/// Valid duplex mode configurations for [`BasicControl`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Duplex {
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
pub enum BasicControlLinkConfig {
    /// Use autonegotiation to configure the link state.
    Autonegotiate {
        /// Restart autonegotiation.
        restart: bool,
    },
    /// Manual configuration of the link state.
    Manual {
        /// The duplex to use.
        duplex: Duplex,
        /// The speed to use.
        speed: LinkSpeed,
    },
}

impl BasicControlLinkConfig {
    /// Check if this link configuration uses autonegotiation.
    pub fn is_autonegotiation(&self) -> bool {
        matches!(self, Self::Autonegotiate { restart: _ })
    }
}

/// The Control Register, containing basic control bits.
///
/// Defined in 37.2.5.1.1.
#[bitsize(16)]
#[derive(FromBits, DebugBits, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(bilge_defmt::FormatBits))]
pub struct BasicControl {
    reserved: u5,
    unidirectional_enable: bool,
    speed_sel_msb: bool,
    /// Enable the collision test signal.
    pub collision_test: bool,
    duplex_mode: DuplexMode,
    restart_autonegotiation: bool,
    /// Electrically isolate the PHY from its (Gigabit) Media Independent
    /// Interface.
    ///
    /// In other words, disconnect the PHY from the pins it shares
    /// with the MAC component connected to it. Setting this bit
    /// _does_ retain access to the PHY over MIIM.
    pub isolate: bool,
    /// Enable Power-Down mode.
    pub power_down: bool,
    autonegotiation_enable: bool,
    speed_sel_lsb: bool,
    /// Enable loopback mode.
    pub loopback: bool,
    /// Perform a reset.
    ///
    /// If set, this bit will stay high until the reset
    /// has completed.
    pub reset: bool,
}

impl BasicControl {
    /// Get the current link configuration.
    pub fn get_link_config(&self) -> BasicControlLinkConfig {
        if self.autonegotiation_enable() {
            BasicControlLinkConfig::Autonegotiate {
                restart: self.restart_autonegotiation(),
            }
        } else {
            let duplex = match self.duplex_mode() {
                DuplexMode::Half => Duplex::Half,
                DuplexMode::Full => Duplex::Full {
                    unidirectional: self.unidirectional_enable(),
                },
            };

            let speed = match (self.speed_sel_lsb(), self.speed_sel_msb()) {
                (false, true) => LinkSpeed::Mbps1000,
                (true, false) => LinkSpeed::Mbps100,
                (false, false) => LinkSpeed::Mbps10,
                (true, true) => panic!("PHY reported invalid speed 0b11"),
            };

            BasicControlLinkConfig::Manual { duplex, speed }
        }
    }

    /// Set the link configuration to `config`.
    pub fn set_link_config(&mut self, config: BasicControlLinkConfig) {
        match config {
            BasicControlLinkConfig::Autonegotiate { restart } => {
                self.set_autonegotiation_enable(true);
                self.set_restart_autonegotiation(restart);
            }
            BasicControlLinkConfig::Manual { duplex, speed } => {
                match duplex {
                    Duplex::Half => self.set_duplex_mode(DuplexMode::Half),
                    Duplex::Full { unidirectional } => {
                        self.set_duplex_mode(DuplexMode::Full);
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

impl Register for BasicControl {
    const ADDRESS: RegisterAddress = RegisterAddress::new(0).unwrap();
}
