//! Leader-follower registers.

use bilge::{bitsize, prelude::*, FromBits};

use crate::registers::Register;

/// The resolved or preference of port type.
#[bitsize(1)]
#[derive(Clone, Copy, FromBits, Debug, PartialEq)]
pub enum PortType {
    /// The port is a leader.
    Leader,
    /// The port is a follower.
    Follower,
}

/// The leader-follower control register.
///
/// This control register defines additional autonegotiation configuration
/// bits, as well as configuration to be used during LEADER-FOLLOWER
/// negotiation.
///
/// Defined in 40.5.1.1.
#[bitsize(16)]
#[derive(Clone, Copy, FromBits, DebugBits)]
pub struct LeaderFollowerControl {
    reserved: u8,
    /// Advertise 1000BASE-T Half Duplex during
    /// autonegotiation.
    pub _1000base_t_hd: bool,
    /// Advertise 1000BASE-T Full Duplex during
    /// autonegotiation.
    pub _1000base_t_fd: bool,
    /// If `!manual_config`, indicate the type of port that
    /// this device should prefer to be.
    pub port_type_preference: PortType,
    /// If `manual_config`, set the port type that this device
    /// should use during negotiation.
    pub config_value: PortType,
    /// Whether manual or LEADER-FOLLOWER negotiation should be
    /// used to determine the [`PortType`] of this device.
    pub manual_config: bool,
    /// The test mode bits.
    pub test_mode: u3,
}

impl Register for LeaderFollowerControl {
    const ADDRESS: u8 = 9;
}

/// The leader-follower status register.
///
/// This status register defines additional autonegotiation status
/// bits, as well as status resolved during LEADER-FOLLOWER
/// negotiation.
///
/// Defined in 40.5.1.1.
#[bitsize(16)]
#[derive(Clone, Copy, FromBits, DebugBits)]
pub struct LeaderFollowerStatus {
    /// The cumulative count of errors detected when the
    /// receiver is receiving idles.
    ///
    /// This field is reset to zero when the register is read.
    pub idle_error_count: u8,
    reserved: u2,
    /// Link partner advertises 1000BASE-T Half Duplex.
    pub _1000base_t_hd: bool,
    /// Link partner advertises 1000BASE-T Full Duplex.
    pub _1000base_t_fd: bool,
    /// Whether the remote receiver is OK, i.e. whether reception
    /// is detected to be reliable for our link partner.
    pub remote_receiver_ok: bool,
    /// Whether the local receiver is OK, i.e. whether reception
    /// is detected to be reliable for this PHY.
    pub local_receiver_ok: bool,
    /// The configuration that the local PHY resolved to.
    pub leader_follower_resolution: PortType,
    /// The combination of the local [`LeaderFollowerControl::config_value`] and
    /// the remote [`LeaderFollowerControl::config_value`] is invalid.
    ///
    /// This field is cleared when the register is read.
    pub leader_follower_config_fault: bool,
}

impl Register for LeaderFollowerStatus {
    const ADDRESS: u8 = 10;
}
