use bilge::{bitsize, prelude::*};

use crate::registers::Register;

#[bitsize(16)]
#[derive(FromBits, DebugBits, Clone, Copy, PartialEq)]
pub struct ExtendedStatus {
    reserved: u12,
    pub _1000base_t_hd: bool,
    pub _1000base_t_fd: bool,
    pub _1000base_x_hd: bool,
    pub _1000base_x_fd: bool,
}

impl Register for ExtendedStatus {
    const ADDRESS: u8 = 15;
}
