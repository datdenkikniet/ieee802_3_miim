//! This module contains definitions of all MIIM registers.
//!
//! These register definitions are based on IEEE 802.3-2022

pub mod auto_negotiation;
mod basic;
pub mod leader_follower;

pub use basic::{
    BasicControl, BasicControlLinkConfig, BasicStatus, Duplex, DuplexMode, ExtendedStatus,
};

/// An MIIM register.
pub trait Register: Into<u16> + From<u16> {
    /// The MIIM address of the register.
    const ADDRESS: u8;
}
