//! Autonegotiation register definitions.
//!
//! Additional bits of information about autonegotiation
//! are found in [`LeaderFollowerStatus`](super::leader_follower::LeaderFollowerStatus)
//! and [`LeaderFollowerControl`](super::leader_follower::LeaderFollowerControl).

use bilge::{bitsize, prelude::*, FromBits};

use crate::registers::{Register, RegisterAddress};

/// The auto negotiation advertisement register, containing (part of)
/// the information that this PHY advertises to its autonegotiation
/// link partner.
///
/// Specified in section 28.2.4.1.3 and 37.2.5.1.3
#[bitsize(16)]
#[derive(FromBits, DebugBits, Clone, Copy)]
pub struct AutonegotiationAdvertisement {
    /// The advertised selector (should be [`Selector::Sel802_3`]).
    pub selector: Selector,
    /// The advertised technology ability.
    pub technology_ability: TechnologyAbility,
    /// Whether this PHY is able to transmit extended
    /// next pages.
    pub extended_next_page: bool,
    /// A remote fault has been detected.
    pub remote_fault: bool,
    reserved: u1,
    /// The advertised next page bit.
    ///
    /// If this is `true`, the local PHY advertised that
    /// it wanted to perform a next-page exchange with its
    /// link partner.
    pub next_page: bool,
}

impl Register for AutonegotiationAdvertisement {
    const ADDRESS: RegisterAddress = RegisterAddress::new(4).unwrap();
}

/// The auto negotiation link partner ability register, containing (part of)
/// the information that is advertised to this PHY by its link partner.
///
/// Defined in section 28.2.4.1.4 and 37.2.5.1.4.
#[bitsize(16)]
#[derive(FromBits, DebugBits, Clone, Copy)]
pub struct AutonegotiationLinkPartnerAbility {
    /// The selector advertised by the link partner (should be [`Selector::Sel802_3`]).
    pub selector: Selector,
    /// The technology ability advertised by the link partner.
    pub technology_ability: TechnologyAbility,
    /// Whether an extended next page was transmitted by the link partner.
    pub extended_next_page: bool,
    /// Whether the link partner detected a local fault.
    pub remote_fault: bool,
    /// Whether the link partner's code word was received
    /// succesfully.
    pub acknowledge: bool,
    /// Whether the link partner wanted to engage in a next page exchange.
    pub next_page: bool,
}

impl Register for AutonegotiationLinkPartnerAbility {
    const ADDRESS: RegisterAddress = RegisterAddress::new(5).unwrap();
}

/// The autonegotiation expansion register.
///
/// Defined in section 28.2.4.1.5 and 37.2.5.1.4.
#[bitsize(16)]
#[derive(FromBits, DebugBits, Clone, Copy)]
pub struct AutonegotiationExpansion {
    /// The link partner is able to perform autonegotiation.
    pub link_partner_autonegotiation_able: bool,
    /// Whether a new page was received from the link partner.
    ///
    /// This field is reset to 0 when on a read of the register.
    pub page_received: bool,
    /// Whether the local device is next-page able.
    pub next_page_able: bool,
    /// Whether the link partner is next-page able.
    pub link_partner_next_page_able: bool,
    /// Whether a fault has been detected using the Parallel
    /// Detection Function.
    pub parallel_detection_fault: bool,
    /// If `true`, an optionally received Next Page is stored in register 8.
    /// If `false`, an optionally received Next Page is stored in register 5.
    pub received_next_page_storage_location: bool,
    /// If `true`, `received_next_page_storage_location` indicates the location
    /// in which an optionally received next page is stored. If `false`, the
    /// location of the next page is implementation-defined.
    pub receive_next_page_location_able: bool,
    reserved: u9,
}

impl Register for AutonegotiationExpansion {
    const ADDRESS: RegisterAddress = RegisterAddress::new(6).unwrap();
}

/// A selector indicating the exact standard that a PHY implements.
///
/// Defined in Annex 28A.
#[bitsize(5)]
#[derive(FromBits, Debug, Clone, Copy, PartialEq)]
#[expect(missing_docs)]
pub enum Selector {
    Sel802_3 = 0b00001,
    Sel802_9a1995 = 0b00010,
    Sel802_5v2001 = 0b00011,
    Sel1394 = 0b00100,
    INCITS = 0b00101,
    #[fallback]
    Reserved,
}

/// The technological abilities of a PHY with the 802.3 selector.
///
/// Defined in Annex 28B.2 and 28D.
#[bitsize(7)]
#[derive(FromBits, DebugBits, Clone, Copy, PartialEq)]
pub struct TechnologyAbility {
    /// Supports 10BASE-T, Half-Duplex.
    pub _10base_t_hd: bool,
    /// Supports 10BASE-T, Full-Duplex.
    pub _10base_t_fd: bool,
    /// Supports 100BASE-TX, Half-Duplex.
    pub _100base_tx_hd: bool,
    /// Supports 100BASE-TX, Full-Duplex.
    pub _100base_tx_fd: bool,
    /// Supports 100BASE-T4.
    pub _100base_t4: bool,
    /// Symmetric pause for full duplex links is supported.
    pub symmetric_pause: bool,
    /// Asymmetric pause for full duplex links is supported.
    pub asymmetric_pause: bool,
}
