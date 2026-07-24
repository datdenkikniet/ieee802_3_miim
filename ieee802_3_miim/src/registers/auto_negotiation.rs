//! Autonegotiation register definitions.
//!
//! Additional bits of information about autonegotiation
//! are found in [`LeaderFollowerStatus`](super::leader_follower::LeaderFollowerStatus)
//! and [`LeaderFollowerControl`](super::leader_follower::LeaderFollowerControl).

use arbitrary_int::u7;

use crate::registers::{Register, RegisterAddress};

/// The auto negotiation advertisement register, containing (part of)
/// the information that this PHY advertises to its autonegotiation
/// link partner.
///
// Specified in section 28.2.4.1.3 and 37.2.5.1.3
#[bitbybit::bitfield(u16, forbid_overlaps, default = 0, defmt_bitfields(feature = "defmt"))]
#[derive(Debug, PartialEq)]
pub struct AutonegotiationAdvertisement {
    /// The advertised selector (should be [`Selector::Sel802_3`]).
    #[bits(0..=4, rw)]
    pub selector: Option<Selector>,
    /// The advertised technology ability.
    #[bits(5..=11, rw)]
    pub technology_ability: TechnologyAbility,
    /// Whether this PHY is able to transmit extended
    /// next pages.
    #[bit(12)]
    pub extended_next_page: bool,
    /// A remote fault has been detected.
    #[bit(13)]
    pub remote_fault: bool,
    /// The advertised next page bit.
    ///
    /// If this is `true`, the local PHY advertised that
    /// it wanted to perform a next-page exchange with its
    /// link partner.
    #[bit(15)]
    pub next_page: bool,
}

from_into!(AutonegotiationAdvertisement);

impl Register for AutonegotiationAdvertisement {
    const ADDRESS: RegisterAddress = RegisterAddress::new(4).unwrap();
}

/// The auto negotiation link partner ability register, containing (part of)
/// the information that is advertised to this PHY by its link partner.
///
// Defined in section 28.2.4.1.4 and 37.2.5.1.4.
#[bitbybit::bitfield(u16, forbid_overlaps, default = 0, defmt_bitfields(feature = "defmt"))]
#[derive(Debug, PartialEq)]
pub struct AutonegotiationLinkPartnerAbility {
    /// The selector advertised by the link partner (should be [`Selector::Sel802_3`]).
    #[bits(0..=4, rw)]
    pub selector: Option<Selector>,
    /// The technology ability advertised by the link partner.
    #[bits(5..=11, rw)]
    pub technology_ability: TechnologyAbility,
    /// Whether an extended next page was transmitted by the link partner.
    #[bit(12)]
    pub extended_next_page: bool,
    /// Whether the link partner detected a local fault.
    #[bit(13)]
    pub remote_fault: bool,
    /// Whether the link partner's code word was received
    /// succesfully.
    #[bit(14)]
    pub acknowledge: bool,
    /// Whether the link partner wanted to engage in a next page exchange.
    #[bit(15)]
    pub next_page: bool,
}

from_into!(AutonegotiationLinkPartnerAbility);

impl Register for AutonegotiationLinkPartnerAbility {
    const ADDRESS: RegisterAddress = RegisterAddress::new(5).unwrap();
}

/// The autonegotiation expansion register.
///
// Defined in section 28.2.4.1.5 and 37.2.5.1.4.
#[bitbybit::bitfield(u16, forbid_overlaps, default = 0, defmt_bitfields(feature = "defmt"))]
#[derive(Debug, PartialEq)]
pub struct AutonegotiationExpansion {
    /// The link partner is able to perform autonegotiation.
    #[bit(0, rw)]
    pub link_partner_autonegotiation_able: bool,
    /// Whether a new page was received from the link partner.
    ///
    /// This field is reset to 0 when the register is read.
    #[bit(1, rw)]
    pub page_received: bool,
    /// Whether the local device is next-page able.
    #[bit(2, rw)]
    pub next_page_able: bool,
    /// Whether the link partner is next-page able.
    #[bit(3, rw)]
    pub link_partner_next_page_able: bool,
    /// Whether a fault has been detected using the Parallel
    /// Detection Function.
    #[bit(4, rw)]
    pub parallel_detection_fault: bool,
    /// If `true`, an optionally received Next Page is stored in register 8.
    /// If `false`, an optionally received Next Page is stored in register 5.
    #[bit(5, rw)]
    pub received_next_page_storage_location: bool,
    /// If `true`, `received_next_page_storage_location` indicates the location
    /// in which an optionally received next page is stored. If `false`, the
    /// location of the next page is implementation-defined.
    #[bit(6, rw)]
    pub receive_next_page_location_able: bool,
}

from_into!(AutonegotiationExpansion);

impl Register for AutonegotiationExpansion {
    const ADDRESS: RegisterAddress = RegisterAddress::new(6).unwrap();
}

/// A selector indicating the exact standard that a PHY implements.
///
// Defined in Annex 28A.
#[bitbybit::bitenum(u5)]
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[expect(missing_docs)]
pub enum Selector {
    Sel802_3 = 0b00001,
    Sel802_9a1995 = 0b00010,
    Sel802_5v2001 = 0b00011,
    Sel1394 = 0b00100,
    INCITS = 0b00101,
}

/// The technological abilities of a PHY with the 802.3 selector.
///
// Defined in Annex 28B.2 and 28D.
#[bitbybit::bitfield(u7, forbid_overlaps, defmt_bitfields(feature = "defmt"))]
#[derive(Debug, PartialEq, Default)]

pub struct TechnologyAbility {
    /// Supports 10BASE-T, Half-Duplex.
    #[bit(0, rw)]
    pub _10base_t_hd: bool,
    /// Supports 10BASE-T, Full-Duplex.
    #[bit(1, rw)]
    pub _10base_t_fd: bool,
    /// Supports 100BASE-TX, Half-Duplex.
    #[bit(2, rw)]
    pub _100base_tx_hd: bool,
    /// Supports 100BASE-TX, Full-Duplex.
    #[bit(3, rw)]
    pub _100base_tx_fd: bool,
    /// Supports 100BASE-T4.
    #[bit(4, rw)]
    pub _100base_t4: bool,
    /// Symmetric pause for full duplex links is supported.
    #[bit(5, rw)]
    pub symmetric_pause: bool,
    /// Asymmetric pause for full duplex links is supported.
    #[bit(6, rw)]
    pub asymmetric_pause: bool,
}
