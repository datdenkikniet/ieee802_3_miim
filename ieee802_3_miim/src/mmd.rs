//! MDIO Manageable Device (MMD, Clause 45) access for MIIM PHYs.

use bilge::{bitsize, prelude::*, FromBits};

use crate::{Miim, RegisterAddress};

/// The access mode for an MMD transaction.
#[bitsize(2)]
#[derive(FromBits, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AccessMode {
    /// Transfer an address to the MMDs address
    /// register.
    Address = 0b00,
    /// Transfer a piece of data to/from a previously
    /// transferred address.
    Data = 0b01,
    /// Transfer a piece of data, and increment
    /// the data pointer afterwards.
    DataPostIncrement = 0b10,
    /// Transfer a piece of data, and increment
    /// the data pointer afterwards if the transaction
    /// is a write..
    DataPostIncrementWrites = 0b11,
}

/// Register 13 & 14
#[bitsize(16)]
#[derive(Clone, Copy, DebugBits, FromBits)]
#[cfg_attr(feature = "defmt", derive(bilge_defmt::FormatBits))]
pub struct MmdAccessControl {
    /// The device address.
    pub device_address: u5,
    reserved: u9,
    /// The access mode of the register.
    pub mode: AccessMode,
}

impl MmdAccessControl {
    const CONTROL_REG: RegisterAddress = RegisterAddress::new(13).unwrap();
    const DATA_ADDRESS_REG: RegisterAddress = RegisterAddress::new(14).unwrap();
}

/// A struct used for MMD access.
pub struct Mmd;

impl Mmd {
    /// Perform an MMD read of addres `reg_address` from device `device_address`
    ///  using the [`Miim::write_raw`] and [`Miim::read_raw`] functionality of `phy`.
    pub fn read<P: Miim>(phy: &mut P, device_address: u5, reg_address: u16) -> u16 {
        let mut mmd_address = MmdAccessControl::new(device_address, AccessMode::Address);
        phy.write_raw(MmdAccessControl::CONTROL_REG, mmd_address.value);
        phy.write_raw(MmdAccessControl::DATA_ADDRESS_REG, reg_address);

        mmd_address.set_mode(AccessMode::Data);
        phy.write_raw(MmdAccessControl::CONTROL_REG, mmd_address.value);
        phy.read_raw(MmdAccessControl::DATA_ADDRESS_REG)
    }

    /// Perform an MMD write at addres `reg_address` on device `device_address`
    ///  using the [`Miim::write_raw`] and [`Miim::read_raw`] functionality of `phy`.
    pub fn write<P: Miim>(phy: &mut P, device_address: u5, reg_address: u16, reg_data: u16) {
        let mut mmd_address = MmdAccessControl::new(device_address, AccessMode::Address);
        phy.write_raw(MmdAccessControl::CONTROL_REG, mmd_address.value);
        phy.write_raw(MmdAccessControl::DATA_ADDRESS_REG, reg_address);

        mmd_address.set_mode(AccessMode::Data);
        phy.write_raw(MmdAccessControl::CONTROL_REG, mmd_address.value);
        phy.write_raw(MmdAccessControl::DATA_ADDRESS_REG, reg_data)
    }
}
