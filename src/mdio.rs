//! This module defines traits and structs used for
//! accessing MIIM over MDIO.

use crate::Miim;

/// A trait used for implementing access to the Media Indepedent
/// Interface of an IEEE 802.3 compatible PHY over MDIO.
pub trait Mdio {
    /// Read MIIM register `reg` from phy `phy` over MDIO.
    fn read(&mut self, phy: u8, reg: u8) -> u16;

    /// Write `data` to MIIM register `reg` on phy `phy` over MDIO.
    fn write(&mut self, phy: u8, reg: u8, data: u16);
}

impl<T> Mdio for &mut T
where
    T: Mdio,
{
    fn read(&mut self, phy: u8, reg: u8) -> u16 {
        <T as Mdio>::read(self, phy, reg)
    }

    fn write(&mut self, phy: u8, reg: u8, data: u16) {
        <T as Mdio>::write(self, phy, reg, data)
    }
}

/// An MIIM compatible PHY that is accessible
/// over MDIO.
#[derive(Debug, Clone)]
pub struct MdioPhy<M: Mdio> {
    /// The MDIO instance used for accessing the bus.
    pub mdio: M,
    /// The address of the MII device on the MDIO bus.
    pub address: u8,
}

impl<M: Mdio> MdioPhy<M> {
    /// Create a struct for managing a device on bus `mdio` at
    /// address `address`.
    pub fn new(mdio: M, address: u8) -> Self {
        Self { mdio, address }
    }
}

impl<M: Mdio> Miim for MdioPhy<M> {
    fn read_raw(&mut self, address: u8) -> u16 {
        let addr = self.address;
        self.mdio.read(addr, address)
    }

    fn write_raw(&mut self, address: u8, value: u16) {
        let addr = self.address;
        self.mdio.write(addr, address, value)
    }
}
