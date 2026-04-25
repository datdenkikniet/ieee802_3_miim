//! This module defines traits and structs used for
//! accessing MIIM over MDIO.

use crate::{Miim, RegisterAddress};

/// An MDIO PHY address.
///
/// The maximum PHY address is 31.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhyAddress(u8);

impl PhyAddress {
    /// Create a new PHY address.
    ///
    /// Returns `None` if `value > 31`.
    pub const fn new(value: u8) -> Option<Self> {
        if value <= 31 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Get the value of this address.
    pub const fn get(&self) -> u8 {
        self.0
    }
}

/// A trait used for implementing access to the Media Indepedent
/// Interface of an IEEE 802.3 compatible PHY over MDIO.
pub trait Mdio {
    /// Read MIIM register `reg` from phy `phy` over MDIO.
    fn read(&mut self, phy: PhyAddress, reg: RegisterAddress) -> u16;

    /// Write `data` to MIIM register `reg` on phy `phy` over MDIO.
    fn write(&mut self, phy: PhyAddress, reg: RegisterAddress, data: u16);
}

impl<T> Mdio for &mut T
where
    T: Mdio,
{
    fn read(&mut self, phy: PhyAddress, reg: RegisterAddress) -> u16 {
        <T as Mdio>::read(self, phy, reg)
    }

    fn write(&mut self, phy: PhyAddress, reg: RegisterAddress, data: u16) {
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
    pub address: PhyAddress,
}

#[cfg(feature = "defmt")]
impl<M: Mdio> defmt::Format for MdioPhy<M> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "MdioPhy {{ address: {} }}", self.address)
    }
}

impl<M: Mdio> MdioPhy<M> {
    /// Create a struct for managing a device on bus `mdio` at
    /// address `address`.
    pub fn new(mdio: M, address: PhyAddress) -> Self {
        Self { mdio, address }
    }
}

impl<M: Mdio> Miim for MdioPhy<M> {
    fn read_raw(&mut self, address: RegisterAddress) -> u16 {
        let addr = self.address;
        self.mdio.read(addr, address)
    }

    fn write_raw(&mut self, address: RegisterAddress, value: u16) {
        let addr = self.address;
        self.mdio.write(addr, address, value)
    }
}
