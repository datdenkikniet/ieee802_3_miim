#![allow(missing_docs)]

use bilge::{bitsize, prelude::*, FromBits};

use crate::{Miim, RegisterAddress};

#[bitsize(2)]
#[derive(FromBits, Debug)]
pub enum AccessMode {
    Address = 0b00,
    Data = 0b01,
    DataPostIncrement = 0b10,
    DataPostIncrementWrites = 0b11,
}

// Register 13
#[bitsize(16)]
#[derive(FromBits, DebugBits)]
pub struct MmdAccessControl {
    pub device_address: u5,
    pub reserved: u9,
    pub mode: AccessMode,
}

impl MmdAccessControl {
    const CONTROL_REG: RegisterAddress = RegisterAddress::new(13).unwrap();
    const DATA_ADDRESS_REG: RegisterAddress = RegisterAddress::new(14).unwrap();
}

pub struct Mmd;

impl Mmd {
    pub fn read<P: Miim>(phy: &mut P, device_address: u5, reg_address: u16) -> u16 {
        let mut mmd_address = MmdAccessControl::new(device_address, AccessMode::Address);
        phy.write_raw(MmdAccessControl::CONTROL_REG, mmd_address.value);
        phy.write_raw(MmdAccessControl::DATA_ADDRESS_REG, reg_address);

        mmd_address.set_mode(AccessMode::Data);
        phy.write_raw(MmdAccessControl::CONTROL_REG, mmd_address.value);
        phy.read_raw(MmdAccessControl::DATA_ADDRESS_REG)
    }

    pub fn write<P: Miim>(phy: &mut P, device_address: u5, reg_address: u16, reg_data: u16) {
        let mut mmd_address = MmdAccessControl::new(device_address, AccessMode::Address);
        phy.write_raw(MmdAccessControl::CONTROL_REG, mmd_address.value);
        phy.write_raw(MmdAccessControl::DATA_ADDRESS_REG, reg_address);

        mmd_address.set_mode(AccessMode::Data);
        phy.write_raw(MmdAccessControl::CONTROL_REG, mmd_address.value);
        phy.write_raw(MmdAccessControl::DATA_ADDRESS_REG, reg_data)
    }
}
