//! SMSC LAN87xxA (LAN8742A, LAN8720A) Ethernet PHYs

use crate::{
    phy::lan87xxa::registers::InterruptReg, registers::Esr, AutoNegotiationAdvertisement,
    ExtendedPhyStatus, Miim, Phy, PhyStatus,
};

use self::registers::{Ssr, PHY_REG_WUCSR};

use super::{AdvancedPhySpeed, PhySpeed, PhyWithSpeed};

/// SMSC LAN8720A Ethernet PHY
pub type LAN8720A<MIIM> = LAN87xxA<MIIM, false>;
/// SMSC LAN8742A Ethernet PHY
pub type LAN8742A<MIIM> = LAN87xxA<MIIM, true>;

/// All interrupt sources supported by this chip
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interrupt {
    /// A page was received during auto negotiation
    AutoNegotiationPageRecvd,
    /// A fault occurred during parallel detection
    ParallelDetectionFault,
    /// The last page transferred during auto negotiation was ACK'd
    AutoNegotiationLpAck,
    /// The link went down
    LinkDown,
    /// A remote fault occurred
    RemoteFault,
    /// Auto negotiation completed
    AutoNegotiationComplete,
    /// PoE Energy was turned on
    EnergyOn,
    /// A Wake on Lan packet was received (only supported on LAN8742A)
    #[cfg(feature = "lan8742a")]
    WoL,
}

impl From<Interrupt> for InterruptReg {
    fn from(int: Interrupt) -> Self {
        match int {
            Interrupt::AutoNegotiationPageRecvd => InterruptReg::INT1_AUTO_NEG_PAGE_RECVD,
            Interrupt::ParallelDetectionFault => InterruptReg::INT2_PARALLELL_DETECTION_FAULT,
            Interrupt::AutoNegotiationLpAck => InterruptReg::INT3_AUTO_NEG_LP_ACK,
            Interrupt::LinkDown => InterruptReg::INT4_LINK_DOWN,
            Interrupt::RemoteFault => InterruptReg::INT5_REMOTE_FAULT,
            Interrupt::AutoNegotiationComplete => InterruptReg::INT6_AUTO_NEG_COMPLETE,
            Interrupt::EnergyOn => InterruptReg::INT7_ENERGYON,
            #[cfg(feature = "lan8742a")]
            Interrupt::WoL => InterruptReg::INT8_WOL,
        }
    }
}

/// An SMSC LAN87XXA Ethernet PHY.
///
/// EXT_WUCSR_CLEAR is used to determine if the "WU CSR" bit
/// in extended registers should be cleared
///
/// This type should not be used directly. Use [`LAN8720A`] or [`LAN8742A`] instead.
#[derive(Debug)]
pub struct LAN87xxA<M: Miim, const HAS_MMD: bool> {
    phy_addr: u8,
    miim: M,
}

impl<M: Miim, const HAS_MMD: bool> LAN87xxA<M, HAS_MMD> {
    /// Create a new LAN87XXA based PHY
    pub fn new(miim: M, phy_addr: u8) -> Self {
        LAN87xxA { miim, phy_addr }
    }

    /// Initialize the PHY
    pub fn phy_init(&mut self) {
        if HAS_MMD {
            // Clear WU CSR
            self.mmd_write(3, PHY_REG_WUCSR, 0);
        }

        self.set_autonegotiation_advertisement(self.best_supported_advertisement());
    }

    /// Get the link speed
    ///
    /// If this returns `None`, some sort of corruption occured, or the PHY is
    /// in an illegal state
    pub fn link_speed(&mut self) -> Option<PhySpeed> {
        let ssr = Ssr::from_bits_truncate(self.read(Ssr::ADDRESS));
        ssr.into()
    }

    /// Check if the link is up
    pub fn link_established(&mut self) -> bool {
        let bsr = self.bsr();
        let ssr = Ssr::from_bits_truncate(self.read(Ssr::ADDRESS));

        // Link established only if it's up, and autonegotiation is completed
        bsr.phy_link_up() && bsr.autoneg_completed() && ssr.contains(Ssr::AUTONEG_DONE)
    }

    /// Block until a link is established
    pub fn block_until_link(&mut self) {
        while !self.link_established() {}
    }

    /// Enable an interrupt
    pub fn enable_interrupt(&mut self, interrupt: Interrupt) {
        let mut reg_val =
            unsafe { InterruptReg::from_bits_unchecked(self.read(InterruptReg::MASK_ADDR)) };
        reg_val.insert(interrupt.into());
        self.write(InterruptReg::MASK_ADDR, reg_val.bits());
    }

    /// Read and clear all interrupts
    pub fn read_and_clear_active_interrupts(
        &mut self,
        interrupt_storage: &mut [Option<Interrupt>; 8],
    ) {
        let reg_val =
            unsafe { InterruptReg::from_bits_unchecked(self.read(InterruptReg::SOURCE_ADDR)) };

        let mut int_idx = 0;
        macro_rules! int {
            ($flag:expr, $int:expr) => {
                #[allow(unused_assignments)]
                if reg_val.contains($flag) {
                    interrupt_storage[int_idx] = Some($int);
                    int_idx += 1;
                }
            };
        }

        int!(
            InterruptReg::INT1_AUTO_NEG_PAGE_RECVD,
            Interrupt::AutoNegotiationPageRecvd
        );
        int!(
            InterruptReg::INT2_PARALLELL_DETECTION_FAULT,
            Interrupt::ParallelDetectionFault
        );
        int!(
            InterruptReg::INT3_AUTO_NEG_LP_ACK,
            Interrupt::AutoNegotiationLpAck
        );
        int!(InterruptReg::INT4_LINK_DOWN, Interrupt::LinkDown);
        int!(InterruptReg::INT5_REMOTE_FAULT, Interrupt::RemoteFault);
        int!(
            InterruptReg::INT6_AUTO_NEG_COMPLETE,
            Interrupt::AutoNegotiationComplete
        );

        int!(InterruptReg::INT7_ENERGYON, Interrupt::EnergyOn);

        #[cfg(feature = "lan8742a")]
        int!(InterruptReg::INT8_WOL, Interrupt::WoL);
    }

    /// Release the underlying [`Miim`]
    pub fn release(self) -> M {
        self.miim
    }
}

impl<M: Miim, const E: bool> Phy<M> for LAN87xxA<M, E> {
    fn best_supported_advertisement(&self) -> AutoNegotiationAdvertisement {
        AutoNegotiationAdvertisement {
            hd_10base_t: true,
            fd_10base_t: true,
            hd_100base_tx: true,
            fd_100base_tx: true,
            base100_t4: false,
            ..Default::default()
        }
    }

    fn get_miim(&mut self) -> &mut M {
        &mut self.miim
    }

    fn get_phy_addr(&self) -> u8 {
        self.phy_addr
    }

    fn status(&mut self) -> PhyStatus {
        crate::PhyStatus {
            base100_t4: false,
            fd_100base_x: true,
            hd_100base_x: true,
            fd_10mbps: true,
            hd_10mbps: true,
            extended_status: false,
            unidirectional: false,
            preamble_suppression: false,
            autonegotiation: true,
            extended_caps: true,
        }
    }

    fn esr(&mut self) -> Option<Esr> {
        None
    }

    fn extended_status(&mut self) -> Option<ExtendedPhyStatus> {
        None
    }
}

impl<M: Miim, const E: bool> PhyWithSpeed<M> for LAN87xxA<M, E> {
    fn get_link_speed(&mut self) -> Option<AdvancedPhySpeed> {
        self.link_speed().map(Into::into)
    }
}

pub mod registers {
    #![allow(missing_docs)]
    //! LAN87xxA registers

    use bitflags::bitflags;

    use crate::phy::PhySpeed;

    pub const PHY_REG_WUCSR: u16 = 0x8010;

    bitflags! {
        pub struct InterruptReg: u16 {
            const INT1_AUTO_NEG_PAGE_RECVD = (1 << 1);
            const INT2_PARALLELL_DETECTION_FAULT = (1 << 2);
            const INT3_AUTO_NEG_LP_ACK = (1 << 3);
            const INT4_LINK_DOWN = (1 << 4);
            const INT5_REMOTE_FAULT = (1 << 5);
            const INT6_AUTO_NEG_COMPLETE = (1 << 6);
            const INT7_ENERGYON = (1 << 7);
            #[cfg(feature = "lan8742a")]
            const INT8_WOL = (1 << 8);
        }

        pub struct Ssr: u16 {
            const AUTONEG_DONE = (1 << 12);
            const FULL_DUPLEX = (0b1 << 4);
            const MBIT100 = (0b1 << 3);
            const MBIT10 = (0b1 << 2);
        }
    }

    impl InterruptReg {
        pub const SOURCE_ADDR: u8 = 29;
        pub const MASK_ADDR: u8 = 30;
    }

    impl Ssr {
        pub const ADDRESS: u8 = 31;
    }

    impl From<Ssr> for Option<PhySpeed> {
        fn from(ssr: Ssr) -> Self {
            let full_duplex = ssr.contains(Ssr::FULL_DUPLEX);
            let mbit_10 = ssr.contains(Ssr::MBIT10);
            let mbit_100 = ssr.contains(Ssr::MBIT100);

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
