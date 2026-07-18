//! MIIM register definitions.
//!
//! Currently, these registers do not provide a `Default` implementation, as picking
//! a sensible default is difficult or impossible. However, all registers _do_ implement
//! [`From<u16>`], which means that they can easily be constructed using `From::from(0)`.
//!
//! The `ID1` and `ID2` registers are not supported directly in this module. To access them,
//! see [`Miim::phy_ident`](crate::Miim::phy_ident).
//!
//! These definitions are based on IEEE 802.3-2022

// 0x0: basic
// 0x1: basic
// 0x2: see lib.rs
// 0x3: see lib.rs
// 0x4: auto_negotiation
// 0x5: auto_negotation
// 0x6: auto_negotation
// 0x7: N/A
// 0x8: N/A
// 0x9: leader_follower
// 0xa: leader_follower
// 0xb: N/A
// 0xc: N/A
// 0xd: MMD
// 0xe: MMD
// 0xf: basic

// Reg 2 and 3 are PHY ident and are handled at a higher
// level.
//
// 12 and 13 are MMD registers

// 4, 5, 6
// 7 and 8 not handled yet
pub mod auto_negotiation;
// 0, 1 and 15
mod basic;
// 9 and 10
pub mod leader_follower;

pub use basic::{
    BasicControl, BasicControlLinkConfig, BasicStatus, Duplex, DuplexConfig, ExtendedStatus,
};

use crate::RegisterAddress;

/// An MIIM register.
pub trait Register: Into<u16> + From<u16> {
    /// The MIIM address of the register.
    const ADDRESS: RegisterAddress;
}
