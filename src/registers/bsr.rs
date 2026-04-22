use bilge::{bitsize, prelude::*, FromBits};

use crate::registers::Register;

/// Register 1, the Basic Status Register
///
/// This register reports part of the PHY's capabilities, and contains status
/// flags.
#[bitsize(16)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, FromBits)]
pub struct BasicStatus {
    pub extended_capabilities: bool,
    pub jabber_detect: bool,
    pub link_status: bool,
    pub autonegotiate_able: bool,
    pub remote_fault: bool,
    pub autonegotiation_complete: bool,
    pub mf_preamble_suppression: bool,
    pub unidirectional_ability: bool,
    pub extended_status: bool,
    pub _100base_t2_hd: bool,
    pub _100base_t2_fd: bool,
    pub _10base_t_hd: bool,
    pub _10base_t_fd: bool,
    pub _100base_x_hd: bool,
    pub _100base_x_fd: bool,
    pub _100base_t4: bool,
}

impl Register for BasicStatus {
    const ADDRESS: u8 = 1;
}
