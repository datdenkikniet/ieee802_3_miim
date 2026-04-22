use bilge::{bitsize, prelude::*, FromBits};

use crate::registers::Register;

#[bitsize(1)]
#[derive(Clone, Copy, FromBits, Debug, PartialEq)]
pub enum PortMultiplicity {
    MultiPort,
    SinglePort,
}

#[bitsize(1)]
#[derive(Clone, Copy, FromBits, Debug, PartialEq)]
pub enum PortType {
    Leader,
    Follower,
}

#[bitsize(16)]
#[derive(Clone, Copy, FromBits, DebugBits)]
pub struct LeaderFollowerControl {
    pub reserved: u8,
    /// Advertise 1000BASE-T Half Duplex during
    /// autonegotiation.
    pub _1000base_t_hd: bool,
    /// Advertise 1000BASE-T Full Duplex during
    /// autonegotiation.
    pub _1000base_t_fd: bool,
    pub port_type: PortMultiplicity,
    pub config_value: PortType,
    pub manual_config: bool,
    pub test_mode: u3,
}

impl Register for LeaderFollowerControl {
    const ADDRESS: u8 = 9;
}

#[bitsize(16)]
#[derive(Clone, Copy, FromBits, DebugBits)]
pub struct LeaderFollowerStatus {
    pub idle_error_count: u8,
    pub reserved: u2,
    /// Link partner advertises 1000BASE-T Half Duplex.
    pub _1000base_t_hd: bool,
    /// Link partner advertises 1000BASE-T Full Duplex.
    pub _1000base_t_fd: bool,
    pub remote_receiver_ok: bool,
    pub local_receiver_ok: bool,
    /// The configuration that the local PHY resolved to.
    pub leader_follower_resolution: PortType,
    pub leader_follower_config_fault: bool,
}

impl Register for LeaderFollowerStatus {
    const ADDRESS: u8 = 10;
}
