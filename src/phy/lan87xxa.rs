//! SMSC LAN87xxA (LAN8742A, LAN8720A) Ethernet PHYs

use crate::{
    phy::lan87xxa::registers::{InterruptMask, InterruptSource},
    Miim, RegisterAddress,
};

use self::registers::PHY_REG_WUCSR;

/// SMSC LAN8720A Ethernet PHY
pub type LAN8720A<MIIM> = LAN87xxA<MIIM, false>;
/// SMSC LAN8742A Ethernet PHY
pub type LAN8742A<MIIM> = LAN87xxA<MIIM, true>;

/// All interrupt sources supported by this chip
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

/// An SMSC LAN87XXA Ethernet PHY.
///
/// EXT_WUCSR_CLEAR is used to determine if the "WU CSR" bit
/// in extended registers should be cleared
///
/// This type should not be used directly. Use [`LAN8720A`] or [`LAN8742A`] instead.
#[derive(Debug)]
pub struct LAN87xxA<M: Miim, const HAS_MMD: bool> {
    /// The MIIM interface used to communicate with this PHY.
    pub miim: M,
}

impl<M: Miim, const HAS_MMD: bool> LAN87xxA<M, HAS_MMD> {
    /// Create a new LAN87XXA based PHY
    pub fn new(miim: M) -> Self {
        LAN87xxA { miim }
    }

    /// Initialize the PHY
    pub fn phy_init(&mut self) {
        if HAS_MMD {
            // Clear WU CSR to enable the interface.
            self.mmd_write(3, PHY_REG_WUCSR, 0);
        }
    }

    /// Enable an interrupt
    pub fn enable_interrupt(&mut self, interrupt: Interrupt) {
        self.modify(|r: &mut InterruptMask| {
            match interrupt {
                Interrupt::AutoNegotiationPageRecvd => r.set_int1_auto_neg_page_recvd(true),
                Interrupt::ParallelDetectionFault => r.set_int2_parallel_detection_fault(true),
                Interrupt::AutoNegotiationLpAck => r.set_int3_auto_neg_lp_ack(true),
                Interrupt::LinkDown => r.set_int4_link_down(true),
                Interrupt::RemoteFault => r.set_int5_remote_fault(true),
                Interrupt::AutoNegotiationComplete => r.set_int6_auto_neg_complete(true),
                Interrupt::EnergyOn => r.set_int7_energyon(true),
                #[cfg(feature = "lan8742a")]
                Interrupt::WoL => r.set_int8_wol(true),
            };
        });
    }

    /// Read and clear all interrupts
    pub fn read_and_clear_active_interrupts(&mut self) -> InterruptSource {
        self.read()
    }
}

impl<M: Miim, const E: bool> Miim for LAN87xxA<M, E> {
    fn read_raw(&mut self, address: RegisterAddress) -> u16 {
        self.miim.read_raw(address)
    }

    fn write_raw(&mut self, address: RegisterAddress, value: u16) {
        self.miim.write_raw(address, value);
    }
}

pub mod registers {
    #![allow(missing_docs)]
    //! LAN87xxA registers

    use bilge::{bitsize, prelude::*};

    use crate::{registers::Register, RegisterAddress};

    pub const PHY_REG_WUCSR: u16 = 0x8010;

    #[bitsize(16)]
    #[derive(FromBits, DebugBits, Clone, Copy)]
    pub struct InterruptSource {
        reserved: bool,
        pub int1_auto_neg_page_recvd: bool,
        pub int2_parallel_detection_fault: bool,
        pub int3_auto_neg_lp_ack: bool,
        pub int4_link_down: bool,
        pub int5_remote_fault: bool,
        pub int6_auto_neg_complete: bool,
        pub int7_energyon: bool,
        #[cfg(feature = "lan8742a")]
        pub int8_wol: bool,
        reserved: u7,
    }

    impl Register for InterruptSource {
        const ADDRESS: RegisterAddress = RegisterAddress::new(29).unwrap();
    }

    #[bitsize(16)]
    #[derive(FromBits, DebugBits, Clone, Copy)]
    pub struct InterruptMask {
        reserved: bool,
        pub int1_auto_neg_page_recvd: bool,
        pub int2_parallel_detection_fault: bool,
        pub int3_auto_neg_lp_ack: bool,
        pub int4_link_down: bool,
        pub int5_remote_fault: bool,
        pub int6_auto_neg_complete: bool,
        pub int7_energyon: bool,
        #[cfg(feature = "lan8742a")]
        pub int8_wol: bool,
        reserved: u7,
    }

    impl Register for InterruptMask {
        const ADDRESS: RegisterAddress = RegisterAddress::new(30).unwrap();
    }
}
