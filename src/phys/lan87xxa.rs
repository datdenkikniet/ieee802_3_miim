//! SMSC LAN87xxA (LAN8742A, LAN8720A) Ethernet PHYs

use core::convert::TryFrom;

use num_enum::{IntoPrimitive, TryFromPrimitive};

/// The link speeds supported by this PHY
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive)]
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

/// SMSC LAN8720A Ethernet PHY
pub type LAN8720A<SMI> = LAN87xxA<SMI, false>;
/// SMSC LAN8742A Ethernet PHY
pub type LAN8742A<SMI> = LAN87xxA<SMI, true>;

use crate::{
    registers::Esr, AutoNegotiationAdvertisement, ExtendedPhyStatus, Mii, Pause, Phy, PhyStatus,
    SelectorField,
};

use self::consts::*;
#[allow(missing_docs, dead_code)]
mod consts {
    const PHY_REG_ANRX: u8 = 0x05;
    const PHY_REG_ANEXP: u8 = 0x06;
    const PHY_REG_ANNPTX: u8 = 0x07;
    const PHY_REG_ANNPRX: u8 = 0x08;
    pub const PHY_REG_SSR: u8 = 0x1F; // Special Status Register
    const PHY_REG_CTL: u8 = 0x0D; // Ethernet PHY Register Control
    const PHY_REG_ADDAR: u8 = 0x0E; // Ethernet PHY Address or Data

    pub const PHY_REG_WUCSR: u16 = 0x8010;

    pub const PHY_REG_SSR_ANDONE: u16 = 1 << 12;
    const PHY_REG_SSR_SPEED: u16 = 0b111 << 2;
    const PHY_REG_SSR_10BASE_HD: u16 = 0b001 << 2;
    const PHY_REG_SSR_10BASE_FD: u16 = 0b101 << 2;
    const PHY_REG_SSR_100BASE_HD: u16 = 0b010 << 2;
    const PHY_REG_SSR_100BASE_FD: u16 = 0b110 << 2;
}

/// An SMSC LAN87XXA Ethernet PHY.
///
/// EXT_WUCSR_CLEAR is used to determine if the "WU CSR" bit
/// in extended registers should be cleared
///
/// This type should not be used directly. Use [`LAN8720A`] or [`LAN8742A`] instead.
pub struct LAN87xxA<S, const EXT_WUCSR_CLEAR: bool> {
    phy_addr: u8,
    smi: S,
}

impl<S: Mii, const EXT_WUCSR_CLEAR: bool> LAN87xxA<S, EXT_WUCSR_CLEAR> {
    /// Create a new LAN87XXA based PHY
    pub fn new(smi: S, phy_addr: u8) -> Self {
        LAN87xxA { smi, phy_addr }
    }

    /// Initialize the PHY
    pub fn phy_init(&mut self) {
        if EXT_WUCSR_CLEAR {
            // Clear WU CSR
            self.mmd_write(3, PHY_REG_WUCSR, 0);
        }

        self.set_autonegotiation(true);
        self.restart_autonegotiation();

        self.set_autonegotiation_advertisement(Self::BEST_SUPPORTED_ADVERTISEMENT);
    }

    /// Get the link speed
    ///
    /// If this returns `None`, some sort of corruption occured, or the PHY is
    /// in an illegal state
    pub fn link_speed(&mut self) -> Option<LinkSpeed> {
        let link_data = self.read(PHY_REG_SSR);
        let link_data = ((link_data >> 2) & 0b111) as u8;
        LinkSpeed::try_from(link_data).ok()
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

    /// Release the underlying [`SerialManagement`]
    pub fn release(self) -> S {
        self.smi
    }
}

impl<M: Mii, const E: bool> Phy<M> for LAN87xxA<M, E> {
    const BEST_SUPPORTED_ADVERTISEMENT: AutoNegotiationAdvertisement =
        AutoNegotiationAdvertisement {
            next_page: false,
            remote_fault: false,
            extended_next_page: false,
            selector_field: SelectorField::Std802_3,
            hd_10base_t: true,
            fd_10base_t: true,
            hd_100base_tx: true,
            fd_100base_tx: true,
            base100_t4: false,
            pause: Pause::NoPause,
        };

    fn get_smi_mut(&mut self) -> &mut M {
        &mut self.smi
    }

    fn get_smi(&self) -> &M {
        &self.smi
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
