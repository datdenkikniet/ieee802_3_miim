#![allow(missing_docs)]
//! This module contains definitions of all MII registers

use bitflags::bitflags;

use crate::{Mii, Phy, PhyStatus};

bitflags! {
    // Register 0
    pub struct Bcr: u16 {
        const RESET = (1 << 15);
        const LOOPBACK = (1 << 14);
        const SPEED_SEL_LSB = (1 << 13);
        const AUTONEG_ENABLE = (1 << 12);
        const POWER_DOWN = (1 << 11);
        const ISOLATE = (1 << 10);
        const RESTART_AUTONEG = (1 << 9);
        const DUPLEX_MODE = (1 << 8);
        const COLLISION_TEST = (1 << 7);
        const SPEED_SEL_MSB = (1 << 6);
        const UNIDIRECTIONAL_ENABLE = (1 << 5);
    }

    // Register 1
    pub struct Bsr: u16 {
        const _100BASET4 = (1 << 15);
        const _100BASEXFD = (1 << 14);
        const _100BASEXHD = (1 << 13);
        const _10MPBSFD = (1 << 12);
        const _10MBPSHD = (1 << 11);
        const _100BASET2FD = (1 << 10);
        const _100BASET2HD = (1 << 9);
        const EXTENDED_STATUS = (1 << 8);
        const UNIDRECTIONAL = (1 << 7);
        const MF_PREAMBLE_SUPPRESSION = (1 << 6);
        const AUTONEG_COMPLETE = (1 << 5);
        const REMOTE_FAULT = (1 << 4);
        const AUTONEG_ABLE = (1 << 3);
        const LINK_STATUS = (1 << 2);
        const JABBER_DETECT = (1 << 1);
        const EXTENDED_CAPABILITIES = (1 << 0);
    }

    // Registers 4 and 5
    pub struct AutoNegCap: u16 {
        const NEXT_PAGE = (1 << 15);
        const REMOTE_FAULT = (1 << 13);
        const EXTENDED_NEXT_PAGE = (1 << 12);

        const _10BASET = (1 << Self::TECH_ABILITY_OFFSET + 6);
        const _10BASETFD = (1 << Self::TECH_ABILITY_OFFSET + 5);
        const _100BASETX = (1 << Self::TECH_ABILITY_OFFSET + 4);
        const _100BASETXFD = (1 << Self::TECH_ABILITY_OFFSET << 3);
        const _100BASET4 = (1 << Self::TECH_ABILITY_OFFSET + 2);
        const PAUSE = (1 << Self::TECH_ABILITY_OFFSET + 1);
        const ASSYMETRIC_PAUSE = (1 << Self::TECH_ABILITY_OFFSET);

        const SEL_802_3 = (0b00001);
        const SEL_802_9_ISLAN_16T = (0b00010);
        const SEL_802_5 = (0b00011);
        const SEL_1394 = (0b00101);
    }

    // Flags located in register 5 or register 8
    pub struct NextPage: u16 {
        const NEXT_PAGE = (1 << 15);
        const ACK = (1 << 14);
        const MESSAGE_PAGE = (1 << 13);
        const ACK2 = (1 << 12);
        const TOGGLE = (1 << 11);
    }

    // Register 6
    pub struct Ane: u16 {
        const RX_NEXT_PAGE_LOC_ABLE = (1 << 6);
        const RX_NEXT_PAGE_LOC = (1 << 5);
        const PARALLEL_DECT_FAULT = (1 << 4);
        const LINK_PARTNER_NEXT_PAGE_ABLE = (1 << 3);
        const NEXT_PAGE_ABLE = (1 << 2);
        const PAGE_RECEIVED = (1 << 1);
        const LINK_PARTNER_AUTONEG_ABLE = (1 << 0);
    }

    // Register 15
    pub struct Esr: u16 {
        const _1000BASEXFD = (1 << 15);
        const _1000BASEXHD = (1 << 14);
        const _1000BASETFD = (1 << 13);
        const _1000BASETHD = (1 << 12);
    }

}

macro_rules! impl_flag {
    ($set:ident, $get:ident, $flag:expr) => {
        pub fn $set(&mut self, value: bool) {
            if value {
                self.insert($flag);
            } else {
                self.remove($flag);
            }
        }

        pub fn $get(&self) -> bool {
            self.contains($flag)
        }
    };
}

impl Bcr {
    pub const ADDRESS: u8 = 0;

    impl_flag!(
        set_unidirectional,
        unidirectional,
        Self::UNIDIRECTIONAL_ENABLE
    );
    impl_flag!(set_collision_test, collision_test, Self::COLLISION_TEST);
    impl_flag!(set_full_duplex, full_duplex, Self::DUPLEX_MODE);
    impl_flag!(set_isolated, isolated, Self::ISOLATE);
    impl_flag!(set_powered_down, powered_down, Self::POWER_DOWN);
    impl_flag!(set_autonegotiation, autonegotiation, Self::AUTONEG_ENABLE);
    impl_flag!(set_loopback, loopback, Self::LOOPBACK);
    impl_flag!(reset, is_resetting, Self::RESET);
}

impl Bsr {
    pub const ADDRESS: u8 = 1;

    pub fn autoneg_completed(&self) -> bool {
        self.contains(Bsr::AUTONEG_COMPLETE)
    }

    pub fn phy_link_up(&self) -> bool {
        self.contains(Bsr::LINK_STATUS)
    }

    pub fn status(&self) -> PhyStatus {
        PhyStatus {
            base100_t4: self.contains(Bsr::_100BASET4),
            fd_100base_x: self.contains(Bsr::_100BASEXFD),
            hd_100base_x: self.contains(Bsr::_100BASEXHD),
            fd_10mbps: self.contains(Bsr::_10MPBSFD),
            hd_10mbps: self.contains(Bsr::_10MBPSHD),
            extended_status: self.contains(Bsr::EXTENDED_STATUS),
            unidirectional: self.contains(Bsr::UNIDRECTIONAL),
            preamble_suppression: self.contains(Bsr::MF_PREAMBLE_SUPPRESSION),
            autonegotiation: self.contains(Bsr::AUTONEG_ABLE),
            extended_caps: self.contains(Bsr::EXTENDED_CAPABILITIES),
        }
    }
}

impl AutoNegCap {
    const TECH_ABILITY_OFFSET: u8 = 5;
    pub const LOCAL_CAP_ADDRESS: u8 = 4;
    pub const PARTNER_CAP_ADDRESS: u8 = 5;
}

impl Ane {
    pub const ADDRESS: u8 = 6;

    /// Determine the location of the next page.
    ///
    /// If `None` is returned, the next page may be located
    /// in either register 5 or 8, but this value must be
    /// provided by the caller
    pub fn next_page_location(&self, default_value: u8) -> u8 {
        if self.contains(Self::RX_NEXT_PAGE_LOC_ABLE) {
            if self.contains(Self::RX_NEXT_PAGE_LOC) {
                8
            } else {
                5
            }
        } else {
            default_value
        }
    }

    pub fn parallel_detection_fault(&self) -> bool {
        self.contains(Self::PARALLEL_DECT_FAULT)
    }

    pub fn page_received(&self) -> bool {
        self.contains(Self::PAGE_RECEIVED)
    }

    pub fn partner_next_page_able(&self) -> bool {
        self.contains(Self::LINK_PARTNER_NEXT_PAGE_ABLE)
    }

    pub fn partner_autoneg_capable(&self) -> bool {
        self.contains(Self::LINK_PARTNER_AUTONEG_ABLE)
    }
}

impl NextPage {
    pub const TRANSMIT_ADDR: u8 = 7;
    pub const DATA_MASK: u16 = 0x7FF;

    impl_flag!(set_ack, ack, Self::ACK);
    impl_flag!(set_ack2, ack2, Self::ACK2);
    impl_flag!(set_message_page, message_page, Self::MESSAGE_PAGE);
    impl_flag!(set_toggle_bit, toggle_bit, Self::TOGGLE);

    pub fn new<M: Mii, P: Phy<M>>(ane: Ane, default_next_page_value: u8, phy: &mut P) -> Self {
        let next_page = ane.next_page_location(default_next_page_value);
        let next_page = phy.read(next_page);

        unsafe { Self::from_bits_unchecked(next_page) }
    }

    pub fn data(&self) -> u16 {
        self.bits & Self::DATA_MASK
    }

    /// Set the data of this NextPage. Only the last
    /// 11 bits of the data are actually used. They are
    /// masked with [`Self::DATA_MASK`]
    pub fn set_data(&mut self, data: u16) {
        self.bits |= data & Self::DATA_MASK;
    }
}

impl Esr {
    pub const ADDRESS: u8 = 15;
}
