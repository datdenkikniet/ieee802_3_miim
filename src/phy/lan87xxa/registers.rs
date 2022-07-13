#![allow(missing_docs)]
//! LAN87xxA registers

use bitflags::bitflags;

bitflags! {
    pub struct InterruptReg: u16 {
        const INT1_AUTO_NEG_PAGE_RECVD = (1 << 1);
        const INT2_PARALLELL_DETECTION_FAULT = (1 << 2);
        const INT3_AUTO_NEG_LP_ACK = (1 << 3);
        const INT4_LINK_DOWN = (1 << 4);
        const INT5_REMOTE_FAULT = (1 << 5);
        const INT6_AUTO_NEG_COMPLETE = (1 << 6);
        const INT7_ENERGYON = (1 << 7);
        #[cfg(feature = "lan8742a")]
        const INT8_WOL = (1 << 8);
    }
}

impl InterruptReg {
    pub const SOURCE_ADDR: u8 = 29;
    pub const MASK_ADDR: u8 = 30;
}
