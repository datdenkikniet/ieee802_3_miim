//! This module contains definitions of all MIIM registers

use crate::{Miim, Phy};

pub use regs::*;
#[allow(missing_docs)]
mod regs {
    use bitflags::bitflags;
    bitflags! {
        /// Register 0, the Base Control Register
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

        /// Register 1, the Base Status Register
        ///
        /// This register reports part of the PHY's capabilities, and contains status
        /// flags.
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

        /// Registers 4 and 5, auto-negotiation capability registers.
        ///
        /// These registers contain information about the abilities that the
        /// local PHY will advertise, and the abilities that the a remote PHY
        /// advertises using auto-negotiation.
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

        /// Register 5 or register 8, the Next Page register
        pub struct NextPage: u16 {
            const NEXT_PAGE = (1 << 15);
            const ACK = (1 << 14);
            const MESSAGE_PAGE = (1 << 13);
            const ACK2 = (1 << 12);
            const TOGGLE = (1 << 11);
        }

        /// Register 6, the Auto-negotiation Expansion Register
        pub struct Ane: u16 {
            const RX_NEXT_PAGE_LOC_ABLE = (1 << 6);
            const RX_NEXT_PAGE_LOC = (1 << 5);
            const PARALLEL_DECT_FAULT = (1 << 4);
            const LINK_PARTNER_NEXT_PAGE_ABLE = (1 << 3);
            const NEXT_PAGE_ABLE = (1 << 2);
            const PAGE_RECEIVED = (1 << 1);
            const LINK_PARTNER_AUTONEG_ABLE = (1 << 0);
        }

        /// Register 15, the Extended Status Register
        pub struct Esr: u16 {
            const _1000BASEXFD = (1 << 15);
            const _1000BASEXHD = (1 << 14);
            const _1000BASETFD = (1 << 13);
            const _1000BASETHD = (1 << 12);
        }

    }

    // This impl lives here because it must access `self.bits`
    impl NextPage {
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
}
macro_rules! impl_flag {
    ($setdoc:literal, $set:ident, $getdoc:literal, $get:ident, $flag:expr) => {
        #[doc = $setdoc]
        pub fn $set(&mut self, value: bool) -> &mut Self {
            if value {
                self.insert($flag);
            } else {
                self.remove($flag);
            }
            self
        }

        #[doc = $getdoc]
        pub fn $get(&self) -> bool {
            self.contains($flag)
        }
    };
}

impl Bcr {
    /// The register address of the BCR register
    pub const ADDRESS: u8 = 0;

    impl_flag!(
        "Configure unidirectional communications mode.",
        set_unidirectional,
        "Get whether the PHY is configured for unidirectional communications mode.",
        unidirectional,
        Self::UNIDIRECTIONAL_ENABLE
    );
    impl_flag!(
        "Enable or disable the collision test signal.",
        set_collision_test,
        "Determine whether the collision test signal is enabled or disabled.",
        collision_test,
        Self::COLLISION_TEST
    );
    impl_flag!(
        "Enable or disable full duplex. This flag is ignored by the PHY if `Self::autonegotiation` is set.",
        set_full_duplex,
        "`true` if full-duplex is enabled. This flag is ignored by the PHY if `Self::autonegotiation` is set.",
        full_duplex,
        Self::DUPLEX_MODE
    );
    impl_flag!(
        "Enable or disable electric isolation mode.",
        set_isolated,
        "Determine whether electric isolation mode is enabled or disabled.",
        isolated,
        Self::ISOLATE
    );
    impl_flag!(
        "Enable or disable power down mode.",
        set_power_down,
        "Determine whether power down is enabled or disabled.",
        power_down,
        Self::POWER_DOWN
    );
    impl_flag!(
        "Enable or disable autonegotiation.",
        set_autonegotiation,
        "Determine if autonegotiation is enabled or disabled.",
        autonegotiation,
        Self::AUTONEG_ENABLE
    );
    impl_flag!(
        "Enable or disable loopback mode.",
        set_loopback,
        "Determine whether loopback mode is enabled or disabled",
        loopback,
        Self::LOOPBACK
    );
    impl_flag!(
        "Reset the PHY.",
        reset,
        "Determine if the PHY is currently resetting",
        is_resetting,
        Self::RESET
    );

    /// Restart the autonegotiation process
    pub fn restart_autonegotiation(&mut self) -> &mut Self {
        self.insert(Self::RESTART_AUTONEG);
        self
    }
}

impl Bsr {
    /// The register address of the BSR
    pub const ADDRESS: u8 = 1;

    /// Check if autonegotiation has completed
    pub fn autoneg_completed(&self) -> bool {
        self.contains(Bsr::AUTONEG_COMPLETE)
    }

    /// Check if the PHY has determined that the link is up.
    pub fn phy_link_up(&self) -> bool {
        self.contains(Bsr::LINK_STATUS)
    }
}

impl AutoNegCap {
    const TECH_ABILITY_OFFSET: u8 = 5;
    /// The address of the local auto-negotiation capabilities register
    pub const LOCAL_CAP_ADDRESS: u8 = 4;
    /// The address of the parter auto-negotiation capabilities register
    pub const PARTNER_CAP_ADDRESS: u8 = 5;
}

impl Ane {
    /// The address of the autonegotiation
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

    /// A parallel detection fault occured,
    pub fn parallel_detection_fault(&self) -> bool {
        self.contains(Self::PARALLEL_DECT_FAULT)
    }

    /// A page was received,
    pub fn page_received(&self) -> bool {
        self.contains(Self::PAGE_RECEIVED)
    }

    /// The link partner is next-page able.
    pub fn partner_next_page_able(&self) -> bool {
        self.contains(Self::LINK_PARTNER_NEXT_PAGE_ABLE)
    }

    /// The link partner is auto-negotiation capable.
    pub fn partner_autoneg_capable(&self) -> bool {
        self.contains(Self::LINK_PARTNER_AUTONEG_ABLE)
    }
}

impl NextPage {
    /// The address of the transmit register.
    pub const TRANSMIT_ADDR: u8 = 7;
    /// The mask used for masking out the data portion of the register.
    pub const DATA_MASK: u16 = 0x7FF;

    impl_flag!(
        "Set the ACK flag.",
        set_ack,
        "Check if the ACK flag is set",
        ack,
        Self::ACK
    );
    impl_flag!(
        "Set the ACK2 flag.",
        set_ack2,
        "Check if the ACK2 flag is set.",
        ack2,
        Self::ACK2
    );
    impl_flag!(
        "Set the message page flag.",
        set_message_page,
        "Check if the message page flag is set.",
        message_page,
        Self::MESSAGE_PAGE
    );
    impl_flag!(
        "Set the toggle bit.",
        set_toggle_bit,
        "Check if the toggle bit is set.",
        toggle_bit,
        Self::TOGGLE
    );

    /// Create a new next page, using the provided Auto-Negotiation Expansion register to
    /// determine the location of the next page, falling back to `default_next_page` if none
    /// is available.
    pub fn new<M: Miim, P: Phy<M>>(ane: Ane, default_next_page: u8, phy: &mut P) -> Self {
        let next_page = ane.next_page_location(default_next_page);
        let next_page = phy.read(next_page);

        unsafe { Self::from_bits_unchecked(next_page) }
    }
}

impl Esr {
    /// The address of the Extended Status Register.
    pub const ADDRESS: u8 = 15;
}
