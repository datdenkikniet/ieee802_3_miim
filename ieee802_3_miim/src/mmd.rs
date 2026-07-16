//! MDIO Manageable Device (MMD, Clause 45) access for MIIM PHYs.

use arbitrary_int::u5;

use crate::{Miim, RegisterAddress};

/// The access mode for an MMD transaction.
#[bitbybit::bitenum(u2, exhaustive = true)]
#[derive(Debug, PartialEq, Eq)]
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
#[bitbybit::bitfield(u16, forbid_overlaps, defmt_bitfields(feature = "defmt"))]
#[derive(Debug, PartialEq)]
pub struct MmdAccessControl {
    /// The device address.
    #[bits(0..=4, rw)]
    pub device_address: u5,
    /// The access mode of the register.
    #[bits(14..=15, rw)]
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
        let mut mmd_address = MmdAccessControl::ZERO
            .with_device_address(device_address)
            .with_mode(AccessMode::Address);
        phy.write_raw(MmdAccessControl::CONTROL_REG, mmd_address.raw_value());
        phy.write_raw(MmdAccessControl::DATA_ADDRESS_REG, reg_address);

        mmd_address.set_mode(AccessMode::Data);
        phy.write_raw(MmdAccessControl::CONTROL_REG, mmd_address.raw_value());
        phy.read_raw(MmdAccessControl::DATA_ADDRESS_REG)
    }

    /// Perform an MMD write at addres `reg_address` on device `device_address`
    ///  using the [`Miim::write_raw`] and [`Miim::read_raw`] functionality of `phy`.
    pub fn write<P: Miim>(phy: &mut P, device_address: u5, reg_address: u16, reg_data: u16) {
        let mut mmd_address = MmdAccessControl::ZERO
            .with_device_address(device_address)
            .with_mode(AccessMode::Address);
        phy.write_raw(MmdAccessControl::CONTROL_REG, mmd_address.raw_value());
        phy.write_raw(MmdAccessControl::DATA_ADDRESS_REG, reg_address);

        mmd_address.set_mode(AccessMode::Data);
        phy.write_raw(MmdAccessControl::CONTROL_REG, mmd_address.raw_value());
        phy.write_raw(MmdAccessControl::DATA_ADDRESS_REG, reg_data)
    }
}
