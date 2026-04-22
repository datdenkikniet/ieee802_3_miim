use bilge::{bitsize, prelude::*, FromBits};

use crate::{registers::Register, LinkSpeed};

/// A configured duplex mode.
#[bitsize(1)]
#[derive(Clone, Copy, Debug, FromBits, PartialEq, Eq)]
pub enum DuplexMode {
    /// Half duplex.
    Half = 0,
    /// Full duplex.
    Full = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duplex {
    Half,
    Full { unidirectional: bool },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BasicControlLinkConfig {
    Autonegotiate { restart: bool },
    Manual { duplex: Duplex, speed: LinkSpeed },
}

impl BasicControlLinkConfig {
    pub fn is_autonegotiation(&self) -> bool {
        matches!(self, Self::Autonegotiate { restart: _ })
    }
}

/// Register 0, the Base Control Register
#[bitsize(16)]
#[derive(FromBits, DebugBits, Clone, Copy, PartialEq)]
pub struct BasicControl {
    reserved: u5,
    pub unidirectional_enable: bool,
    speed_sel_msb: bool,
    pub collision_test: bool,
    duplex_mode: DuplexMode,
    restart_autonegotiation: bool,
    pub isolate: bool,
    pub power_down: bool,
    autonegotiation_enable: bool,
    speed_sel_lsb: bool,
    pub loopback: bool,
    pub reset: bool,
}

impl BasicControl {
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
    const ADDRESS: u8 = 0;
}
