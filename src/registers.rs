//! This module contains definitions of all MIIM registers

#![allow(missing_docs)]

pub mod auto_negotiation;
mod bcr;
mod bsr;
mod extended_status;
pub mod leader_follower;

pub use bcr::{BasicControl, BasicControlLinkConfig, Duplex, DuplexMode};
pub use bsr::BasicStatus;
pub use extended_status::ExtendedStatus;

pub trait Register: Into<u16> + From<u16> {
    const ADDRESS: u8;
}
