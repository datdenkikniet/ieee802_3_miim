//! This module defines traits and structs used for access to
//! Media Independent Interface

/// A trait used for implementing access to the Media Indepedent
/// Interface of an IEEE 802.3 compatible PHY.
pub trait Mii {
    /// Read an SMI register
    fn read(&self, phy: u8, reg: u8) -> u16;
    /// Write an SMI register
    fn write(&mut self, phy: u8, reg: u8, data: u16);
}
