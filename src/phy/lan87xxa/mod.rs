//! SMSC LAN87xxA (LAN8742A, LAN8720A) Ethernet PHYs

pub mod registers;

use crate::{registers::Esr, AutoNegotiationAdvertisement, ExtendedPhyStatus, Mii, Phy, PhyStatus};

use self::{consts::*, registers::InterruptReg};
mod consts {

    pub const PHY_REG_SSR: u8 = 0x1F; // Special Status Register
    pub const PHY_REG_WUCSR: u16 = 0x8010;
    pub const PHY_REG_SSR_ANDONE: u16 = 1 << 12;
}

/// SMSC LAN8720A Ethernet PHY
pub type LAN8720A<MII> = LAN87xxA<MII, false>;
/// SMSC LAN8742A Ethernet PHY
pub type LAN8742A<MII> = LAN87xxA<MII, true>;

/// The link speeds supported by this PHY
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum LinkSpeed {
    /// 10BaseT - Half duplex
    BaseT10HalfDuplex = 0b001,
    /// 10BaseT - Full duplex
    BaseT10FullDuplex = 0b101,
    /// 100BaseT - Half duplex
    BaseT100HalfDuplex = 0b010,
    /// 100BaseT - Full duplex
    BaseT100FullDuplex = 0b110,
}

impl LinkSpeed {
    fn from_u8(val: u8) -> Option<Self> {
        let speed = match val {
            0b001 => LinkSpeed::BaseT10HalfDuplex,
            0b101 => LinkSpeed::BaseT10FullDuplex,
            0b010 => LinkSpeed::BaseT100HalfDuplex,
            0b110 => LinkSpeed::BaseT100FullDuplex,
            _ => return None,
        };
        Some(speed)
    }
}

/// All interrupt sources supported by this chip
#[derive(Debug, Clone, Copy)]
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
pub struct LAN87xxA<M: Mii, const HAS_MMD: bool> {
    phy_addr: u8,
    mii: M,
}

impl<M: Mii, const HAS_MMD: bool> LAN87xxA<M, HAS_MMD> {
    /// Create a new LAN87XXA based PHY
    pub fn new(mii: M, phy_addr: u8) -> Self {
        LAN87xxA { mii, phy_addr }
    }

    /// Initialize the PHY
    pub fn phy_init(&mut self) {
        if HAS_MMD {
            // Clear WU CSR
            self.mmd_write(3, PHY_REG_WUCSR, 0);
        }

        self.set_autonegotiation_advertisement(self.best_supported_advertisement());
        self.modify_bcr(|bcr| {
            bcr.set_autonegotiation(true).restart_autonegotiation();
        })
    }

    /// Get the link speed
    ///
    /// If this returns `None`, some sort of corruption occured, or the PHY is
    /// in an illegal state
    pub fn link_speed(&mut self) -> Option<LinkSpeed> {
        let link_data = self.read(PHY_REG_SSR);
        let link_data = ((link_data >> 2) & 0b111) as u8;
        LinkSpeed::from_u8(link_data)
    }

    /// Check if the link is up
    pub fn link_established(&mut self) -> bool {
        let bsr = self.bsr();
        let ssr = self.read(PHY_REG_SSR);

        // Link established only if it's up, and autonegotiation is completed
        !(!bsr.phy_link_up() || !bsr.autoneg_completed() || ssr & PHY_REG_SSR_ANDONE == 0)
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

    /// Release the underlying [`Mii`]
    pub fn release(self) -> M {
        self.mii
    }
}

impl<M: Mii, const E: bool> Phy<M> for LAN87xxA<M, E> {
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

    fn get_mii_mut(&mut self) -> &mut M {
        &mut self.mii
    }

    fn get_mii(&self) -> &M {
        &self.mii
    }

    fn get_phy_addr(&self) -> u8 {
        self.phy_addr
    }

    fn status(&self) -> PhyStatus {
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

    fn esr(&self) -> Option<Esr> {
        None
    }

    fn extended_status(&self) -> Option<ExtendedPhyStatus> {
        None
    }
}
