//! This module defines traits and structs used for access to
//! Media Independent Interface

/// A trait used for implementing access to the Media Indepedent
/// Interface of an IEEE 802.3 compatible PHY.
pub trait Miim {
    /// Read an MII register
    ///
    /// This function receives `&mut self` because it is likely
    /// for implementations to expect to have unique access to underlying
    /// hardware elements (such as pins, or the MAC itself).
    fn read(&mut self, phy: u8, reg: u8) -> u16;

    /// Write to an MII register
    fn write(&mut self, phy: u8, reg: u8, data: u16);
}
