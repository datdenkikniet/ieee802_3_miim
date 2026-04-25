//! Phy implementation for the Microchip KSZ8081R

use crate::{Miim, RegisterAddress};

/// A KSZ8081R
#[derive(Debug)]
pub struct KSZ8081R<MIIM: Miim> {
    /// The MIIM interface used to communicate with this PHY.
    pub miim: MIIM,
}

impl<MIIM: Miim> KSZ8081R<MIIM> {
    const INTERRUPT_REG: RegisterAddress = RegisterAddress::new(0x1B).unwrap();
    const INTERRUPT_REG_EN_LINK_UP: u16 = 1 << 8;
    const INTERRUPT_REG_EN_LINK_DOWN: u16 = 1 << 10;

    /// A mask for determining if the Link Up Interrupt occurred
    pub const INTERRUPT_REG_INT_LINK_UP: u16 = 1 << 0;
    /// A mask for determining if the Link Down Interrupt occurred
    pub const INTERRUPT_REG_INT_LINK_DOWN: u16 = 1 << 2;

    /// Create a new Ksz8081r backed by the given `miim`,
    pub fn new(miim: MIIM) -> Self {
        Self { miim }
    }

    /// Enable the link up and link down interrupts
    pub fn interrupt_enable(&mut self) {
        self.write_raw(
            Self::INTERRUPT_REG,
            Self::INTERRUPT_REG_EN_LINK_UP | Self::INTERRUPT_REG_EN_LINK_DOWN,
        );
    }

    /// Get the value of the interrupt register.
    ///
    /// Use [`Self::INTERRUPT_REG_INT_LINK_UP`] and [`Self::INTERRUPT_REG_INT_LINK_DOWN`]
    /// to determine the type of interrupt that occurred
    pub fn get_interrupt_reg_val(&mut self) -> u16 {
        self.read_raw(Self::INTERRUPT_REG)
    }
}

impl<MIIM: Miim> Miim for KSZ8081R<MIIM> {
    fn read_raw(&mut self, address: RegisterAddress) -> u16 {
        self.miim.read_raw(address)
    }

    fn write_raw(&mut self, address: RegisterAddress, value: u16) {
        self.miim.write_raw(address, value);
    }
}
