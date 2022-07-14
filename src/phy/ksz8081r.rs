//! Phy implementation for the Microchip KSZ8081R

use crate::{registers::Esr, AutoNegotiationAdvertisement, ExtendedPhyStatus, Mii, Phy};

/// A KSZ8081R
pub struct Ksz8081r<MII: Mii> {
    phy_addr: u8,
    mii: MII,
}

impl<MII: Mii> Ksz8081r<MII> {
    const INTERRUPT_REG: u8 = 0x1B;
    const INTERRUPT_REG_EN_LINK_UP: u16 = 1 << 8;
    const INTERRUPT_REG_EN_LINK_DOWN: u16 = 1 << 10;

    /// A mask for determining if the Link Up Interrupt occurred
    pub const INTERRUPT_REG_INT_LINK_UP: u16 = 1 << 0;
    /// A mask for determining if the Link Down Interrupt occurred
    pub const INTERRUPT_REG_INT_LINK_DOWN: u16 = 1 << 2;

    /// Create a new Ksz8081r at `phy_addr`, backed by the given `mii`,
    pub fn new(phy_addr: u8, mii: MII) -> Self {
        Self { phy_addr, mii }
    }

    /// Enable the link up and link down interrupts
    pub fn interrupt_enable(&mut self) {
        self.write(
            Self::INTERRUPT_REG,
            Self::INTERRUPT_REG_EN_LINK_UP | Self::INTERRUPT_REG_EN_LINK_DOWN,
        );
    }

    /// Get the value of the interrupt register.
    ///
    /// Use [`Self::INTERRUPT_REG_INT_LINK_UP`] and [`Self::INTERRUPT_REG_INT_LINK_DOWN`]
    /// to determine the type of interrupt that occurred
    pub fn get_interrupt_reg_val(&self) -> u16 {
        self.read(Self::INTERRUPT_REG)
    }
}

impl<MII: Mii> Phy<MII> for Ksz8081r<MII> {
    fn best_supported_advertisement(&self) -> AutoNegotiationAdvertisement {
        AutoNegotiationAdvertisement {
            hd_10base_t: true,
            fd_10base_t: true,
            hd_100base_tx: true,
            fd_100base_tx: true,
            base100_t4: true,
            ..Default::default()
        }
    }

    fn get_mii_mut(&mut self) -> &mut MII {
        &mut self.mii
    }

    fn get_mii(&self) -> &MII {
        &self.mii
    }

    fn get_phy_addr(&self) -> u8 {
        self.phy_addr
    }

    fn esr(&self) -> Option<Esr> {
        None
    }

    fn extended_status(&self) -> Option<ExtendedPhyStatus> {
        None
    }
}
