//! Phy implementation for the Microchip KSZ8081R

use crate::{registers::Esr, AutoNegotiationAdvertisement, ExtendedPhyStatus, Miim, Phy};

use self::registers::PhyControl1;

use super::{AdvancedPhySpeed, PhySpeed, PhyWithSpeed};

/// A KSZ8081R
#[derive(Debug)]
pub struct KSZ8081R<MIIM: Miim> {
    phy_addr: u8,
    miim: MIIM,
}

impl<MIIM: Miim> KSZ8081R<MIIM> {
    const INTERRUPT_REG: u8 = 0x1B;
    const INTERRUPT_REG_EN_LINK_UP: u16 = 1 << 8;
    const INTERRUPT_REG_EN_LINK_DOWN: u16 = 1 << 10;

    /// A mask for determining if the Link Up Interrupt occurred
    pub const INTERRUPT_REG_INT_LINK_UP: u16 = 1 << 0;
    /// A mask for determining if the Link Down Interrupt occurred
    pub const INTERRUPT_REG_INT_LINK_DOWN: u16 = 1 << 2;

    /// Create a new Ksz8081r at `phy_addr`, backed by the given `miim`,
    pub fn new(miim: MIIM, phy_addr: u8) -> Self {
        Self { phy_addr, miim }
    }

    /// Enable the link up and link down interrupts
    pub fn interrupt_enable(&mut self) {
        self.write(
            Self::INTERRUPT_REG,
            Self::INTERRUPT_REG_EN_LINK_UP | Self::INTERRUPT_REG_EN_LINK_DOWN,
        );
    }

    /// Get the link speed at which the PHY is currently operating
    pub fn link_speed(&mut self) -> Option<PhySpeed> {
        let phy_ctrl1 = PhyControl1::from_bits_truncate(self.read(PhyControl1::ADDRESS));
        phy_ctrl1.into()
    }

    /// Get the value of the interrupt register.
    ///
    /// Use [`Self::INTERRUPT_REG_INT_LINK_UP`] and [`Self::INTERRUPT_REG_INT_LINK_DOWN`]
    /// to determine the type of interrupt that occurred
    pub fn get_interrupt_reg_val(&mut self) -> u16 {
        self.read(Self::INTERRUPT_REG)
    }

    /// Check whether a link is established or not
    pub fn link_established(&mut self) -> bool {
        self.autoneg_completed() && self.phy_link_up()
    }

    /// Release the underlying [`Miim`]
    pub fn release(self) -> MIIM {
        self.miim
    }
}

impl<MIIM: Miim> Phy<MIIM> for KSZ8081R<MIIM> {
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

    fn get_miim(&mut self) -> &mut MIIM {
        &mut self.miim
    }

    fn get_phy_addr(&self) -> u8 {
        self.phy_addr
    }

    fn esr(&mut self) -> Option<Esr> {
        None
    }

    fn extended_status(&mut self) -> Option<ExtendedPhyStatus> {
        None
    }
}

impl<MIIM: Miim> PhyWithSpeed<MIIM> for KSZ8081R<MIIM> {
    fn get_link_speed(&mut self) -> Option<AdvancedPhySpeed> {
        self.link_speed().map(Into::into)
    }
}

#[allow(missing_docs)]
pub mod registers {
    use bitflags::bitflags;

    use crate::phy::PhySpeed;

    bitflags! {
        pub struct PhyControl1: u16 {
            const ENABLE_PAUSE = (1 << 9);
            const LINK_STATUS = (1 << 8);
            const POLARITY_STATUS = (1 << 7);
            const MID_MIDX_STATE = (1 << 5);
            const ENERGY_DETECT = (1 << 4);
            const PHY_ISOLATE = (1 << 3);
            const FULL_DUPLEX = (0b1 << 2);
            const MBIT100 = (0b1 << 1);
            const MBIT10 = (0b1 << 0);
        }
    }

    impl PhyControl1 {
        pub const ADDRESS: u8 = 0x1E;
    }

    impl From<PhyControl1> for Option<PhySpeed> {
        fn from(ctrl: PhyControl1) -> Self {
            let full_duplex = ctrl.contains(PhyControl1::FULL_DUPLEX);
            let mbit_10 = ctrl.contains(PhyControl1::MBIT10);
            let mbit_100 = ctrl.contains(PhyControl1::MBIT100);

            // allow collapsible else/if for clearer semantics
            #[allow(clippy::collapsible_else_if)]
            let speed = if full_duplex {
                if mbit_10 {
                    PhySpeed::FullDuplexBase10T
                } else if mbit_100 {
                    PhySpeed::FullDuplexBase100Tx
                } else {
                    return None;
                }
            } else {
                if mbit_10 {
                    PhySpeed::HalfDuplexBase10T
                } else if mbit_100 {
                    PhySpeed::HalfDuplexBase100Tx
                } else {
                    return None;
                }
            };

            Some(speed)
        }
    }
}
