//! Leader-follower registers.

use arbitrary_int::u3;

use crate::registers::{Register, RegisterAddress};

/// The resolved or preference of port type.
#[bitbybit::bitenum(u1, exhaustive = true)]
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PortType {
    /// The port is a follower.
    Follower = 0,
    /// The port is a leader.
    Leader = 1,
}

/// The leader-follower control register.
///
/// This control register defines additional autonegotiation configuration
/// bits, as well as configuration to be used during LEADER-FOLLOWER
/// negotiation.
///
// Defined in 40.5.1.1.
#[bitbybit::bitfield(u16, forbid_overlaps, default = 0, defmt_bitfields(feature = "defmt"))]
#[derive(Debug, PartialEq)]
pub struct LeaderFollowerControl {
    /// Advertise 1000BASE-T Half Duplex during
    /// autonegotiation.
    #[bit(8, rw)]
    pub _1000base_t_hd: bool,
    /// Advertise 1000BASE-T Full Duplex during
    /// autonegotiation.
    #[bit(9, rw)]
    pub _1000base_t_fd: bool,
    /// If `!manual_config`, indicate the type of port that
    /// this device should prefer to be.
    #[bit(10, rw)]
    pub port_type_preference: PortType,
    /// If `manual_config`, set the port type that this device
    /// should use during negotiation.
    #[bit(11, rw)]
    pub config_value: PortType,
    /// Whether manual or LEADER-FOLLOWER negotiation should be
    /// used to determine the [`PortType`] of this device.
    #[bit(12, rw)]
    pub manual_config: bool,
    /// The test mode bits.
    #[bits(13..=15, rw)]
    pub test_mode: u3,
}

from_into!(LeaderFollowerControl);

impl Register for LeaderFollowerControl {
    const ADDRESS: RegisterAddress = RegisterAddress::new(9).unwrap();
}

/// The leader-follower status register.
///
/// This status register defines additional autonegotiation status
/// bits, as well as status resolved during LEADER-FOLLOWER
/// negotiation.
///
// Defined in 40.5.1.1.
#[bitbybit::bitfield(u16, forbid_overlaps, default = 0, defmt_bitfields(feature = "defmt"))]
#[derive(Debug, PartialEq)]
pub struct LeaderFollowerStatus {
    /// The cumulative count of errors detected when the
    /// receiver is receiving idles.
    ///
    /// This field is reset to zero when the register is read.
    #[bits(0..=7, rw)]
    pub idle_error_count: u8,
    /// Link partner advertises 1000BASE-T Half Duplex.
    #[bit(10, rw)]
    pub _1000base_t_hd: bool,
    /// Link partner advertises 1000BASE-T Full Duplex.
    #[bit(11, rw)]
    pub _1000base_t_fd: bool,
    /// Whether the remote receiver is OK, i.e. whether reception
    /// is detected to be reliable for our link partner.
    #[bit(12, rw)]
    pub remote_receiver_ok: bool,
    /// Whether the local receiver is OK, i.e. whether reception
    /// is detected to be reliable for this PHY.
    #[bit(13, rw)]
    pub local_receiver_ok: bool,
    /// The configuration that the local PHY resolved to.
    #[bit(14, rw)]
    pub leader_follower_resolution: PortType,
    /// The combination of the local [`LeaderFollowerControl::config_value`] and
    /// the remote [`LeaderFollowerControl::config_value`] is invalid.
    ///
    /// This field is cleared when the register is read.
    #[bit(15, rw)]
    pub leader_follower_config_fault: bool,
}

from_into!(LeaderFollowerStatus);

impl Register for LeaderFollowerStatus {
    const ADDRESS: RegisterAddress = RegisterAddress::new(10).unwrap();
}
