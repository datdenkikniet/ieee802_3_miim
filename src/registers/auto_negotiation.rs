use bilge::{bitsize, prelude::*, FromBits};

use crate::registers::Register;

#[bitsize(16)]
#[derive(FromBits, DebugBits, Clone, Copy)]
pub struct AutonegotiationAdvertisement {
    pub selector: Selector,
    pub technology_ability: TechnologyAbility,
    pub extended_next_page: bool,
    pub remote_fault: bool,
    pub reserved: u1,
    pub next_page: bool,
}

impl Register for AutonegotiationAdvertisement {
    const ADDRESS: u8 = 4;
}

#[bitsize(16)]
#[derive(FromBits, DebugBits, Clone, Copy)]
pub struct AutonegotiationLinkPartnerAbility {
    pub selector: Selector,
    pub technology_ability: TechnologyAbility,
    pub extended_next_page: bool,
    pub remote_fault: bool,
    pub acknowledge: bool,
    pub next_page: bool,
}

impl Register for AutonegotiationLinkPartnerAbility {
    const ADDRESS: u8 = 5;
}

#[bitsize(16)]
#[derive(FromBits, DebugBits, Clone, Copy)]
pub struct AutonegotiationExpansion {
    pub link_partner_autonegotiation_able: bool,
    pub page_received: bool,
    pub next_page_able: bool,
    pub link_partner_next_page_able: bool,
    pub parallel_detection_fault: bool,
    received_next_page_storage_location: bool,
    receive_next_page_location_able: bool,
    pub reserved: u9,
}

impl Register for AutonegotiationExpansion {
    const ADDRESS: u8 = 6;
}

#[bitsize(5)]
#[derive(FromBits, Debug, Clone, Copy, PartialEq)]
pub enum Selector {
    Sel802_3 = 0b00001,
    Sel802_9a1995 = 0b00010,
    Sel802_5v2001 = 0b00011,
    Sel1394 = 0b00100,
    INCITS = 0b00101,
    #[fallback]
    Reserved,
}

#[bitsize(7)]
#[derive(FromBits, DebugBits, Clone, Copy, PartialEq)]
pub struct TechnologyAbility {
    pub _10base_t_hd: bool,
    pub _10base_t_fd: bool,
    pub _100base_tx_hd: bool,
    pub _100base_tx_fd: bool,
    pub _100base_t4: bool,
    /// Symmetric pause for full duplex links is supported.
    pub symmetric_pause: bool,
    /// Asymmetric pause for full duplex links is supported.
    pub asymmetric_pause: bool,
}
