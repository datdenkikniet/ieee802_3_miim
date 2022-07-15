#![allow(missing_docs)]

use bitflags::bitflags;

use crate::{Miim, Phy};

bitflags! {
    // Register 13
    pub struct MmdAddress: u16 {
        const ADDRESS = (0b00 << 14);
        const DATA_NO_POSTINC = (0b01 << 14);
        const DATA_POSTINC_RW = (0b10 << 14);
        const DATA_POSTINC_W = (0b11 << 14);
    }
}

impl MmdAddress {
    pub const CONTROL_ADDRESS: u8 = 13;
    pub const DATA_ADRESS_ADDRESS: u8 = 14;

    pub const DEVAD_MASK: u16 = 0b11111;

    pub fn device_address(device_address: u8) -> Self {
        let mut me = Self::ADDRESS;
        me.set_device_address(device_address);
        me
    }

    /// Set the device address value.
    ///
    /// The address is masked with [`Self::DEVAD_MASK`] to ensure
    /// that it is valid
    pub fn set_device_address(&mut self, address: u8) {
        self.bits |= address as u16 & Self::DEVAD_MASK
    }
}

pub struct Mmd;

impl Mmd {
    pub fn read<M: Miim, P: Phy<M>>(phy: &mut P, device_address: u8, reg_address: u16) -> u16 {
        let mut mmd_address = MmdAddress::device_address(device_address);
        phy.write(MmdAddress::CONTROL_ADDRESS, mmd_address.bits());
        phy.write(MmdAddress::DATA_ADRESS_ADDRESS, reg_address);

        mmd_address.remove(MmdAddress::ADDRESS);
        mmd_address.insert(MmdAddress::DATA_NO_POSTINC);
        phy.write(MmdAddress::CONTROL_ADDRESS, mmd_address.bits());
        phy.read(MmdAddress::DATA_ADRESS_ADDRESS)
    }

    pub fn write<M: Miim, P: Phy<M>>(
        phy: &mut P,
        device_address: u8,
        reg_address: u16,
        reg_data: u16,
    ) {
        let mut mmd_address = MmdAddress::device_address(device_address);
        phy.write(MmdAddress::CONTROL_ADDRESS, mmd_address.bits());
        phy.write(MmdAddress::DATA_ADRESS_ADDRESS, reg_address);

        mmd_address.remove(MmdAddress::ADDRESS);
        mmd_address.insert(MmdAddress::DATA_NO_POSTINC);
        phy.write(MmdAddress::CONTROL_ADDRESS, mmd_address.bits());
        phy.write(MmdAddress::DATA_ADRESS_ADDRESS, reg_data);
    }
}
