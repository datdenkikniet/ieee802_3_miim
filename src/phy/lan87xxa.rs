//! SMSC LAN87xxA (LAN8742A, LAN8720A) Ethernet PHYs

use crate::{registers::Esr, AutoNegotiationAdvertisement, ExtendedPhyStatus, Mii, Phy, PhyStatus};

/// SMSC LAN8720A Ethernet PHY
pub type LAN8720A<SMI> = LAN87xxA<SMI, false>;
/// SMSC LAN8742A Ethernet PHY
pub type LAN8742A<SMI> = LAN87xxA<SMI, true>;

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

use self::consts::*;
mod consts {

    pub const PHY_REG_SSR: u8 = 0x1F; // Special Status Register
    pub const PHY_REG_WUCSR: u16 = 0x8010;
    pub const PHY_REG_SSR_ANDONE: u16 = 1 << 12;
}

/// An SMSC LAN87XXA Ethernet PHY.
///
/// EXT_WUCSR_CLEAR is used to determine if the "WU CSR" bit
/// in extended registers should be cleared
///
/// This type should not be used directly. Use [`LAN8720A`] or [`LAN8742A`] instead.
pub struct LAN87xxA<M: Mii, const EXT_WUCSR_CLEAR: bool> {
    phy_addr: u8,
    smi: M,
}

impl<M: Mii, const EXT_WUCSR_CLEAR: bool> LAN87xxA<M, EXT_WUCSR_CLEAR> {
    /// Create a new LAN87XXA based PHY
    pub fn new(mii: M, phy_addr: u8) -> Self {
        LAN87xxA { smi: mii, phy_addr }
    }

    /// Initialize the PHY
    pub fn phy_init(&mut self) {
        if EXT_WUCSR_CLEAR {
            // Clear WU CSR
            self.mmd_write(3, PHY_REG_WUCSR, 0);
        }

        self.set_autonegotiation(true);
        self.set_autonegotiation_advertisement(self.best_supported_advertisement());
        self.restart_autonegotiation();
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

    /// Release the underlying [`Mii`]
    pub fn release(self) -> M {
        self.smi
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
        &mut self.smi
    }

    fn get_mii(&self) -> &M {
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
