#![cfg_attr(not(test), no_std)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod mdio;

pub mod mmd;
use mmd::Mmd;

pub mod registers;
use registers::*;

use crate::registers::{
    auto_negotiation::AutonegotiationAdvertisement, leader_follower::LeaderFollowerControl,
};

mod link_state;
pub use link_state::{GetLinkStateProcess, LinkStateError};

/// A MIIM register address.
///
/// The maximum MIIM register address is 31.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RegisterAddress(u8);

impl RegisterAddress {
    /// Create a new register address.
    ///
    /// Returns `None` if `value > 31`.
    pub const fn new(value: u8) -> Option<Self> {
        if value < 32 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Get the value of this address.
    pub const fn get(&self) -> u8 {
        self.0
    }
}

/// The state of an (R)(G)MII link.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LinkState {
    /// The speed of the link.
    pub speed: LinkSpeed,
    /// The duplex mode of the link.
    pub duplex: Duplex,
}

/// All link speeds that may be supported by a normal (R)(G)MII PHY.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LinkSpeed {
    /// 10 Mbps
    Mbps10,
    /// 100 Mbps
    Mbps100,
    /// 1000 Mbps
    Mbps1000,
}

/// The PHY IDENT of a [`Miim`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PhyIdent(u16, u16);

impl PhyIdent {
    /// Create a new PhyIdent
    ///
    /// `phy_ident_1` should be the contents of the ID1 register
    /// of the PHY, and `phy_ident_2` should be the contents of the
    /// ID2 register of the PHY.
    pub const fn new(phy_ident_1: u16, phy_ident_2: u16) -> Self {
        Self(phy_ident_1, phy_ident_2)
    }

    /// The raw values of this PhyIdent
    pub const fn raw(&self) -> (u16, u16) {
        (self.0, self.1)
    }

    /// The raw value of this PhyIdent, as u32
    pub const fn raw_u32(&self) -> u32 {
        (self.0 as u32) << 16 | (self.1 as u32)
    }

    /// The OUI of this PhyIdent
    pub const fn oui(&self) -> u32 {
        (self.0 as u32) << 6 & (self.1 as u32) >> 10
    }

    /// The model number of this PhyIdent
    pub const fn model_number(&self) -> u8 {
        (self.1 >> 4) as u8 & 0x3F
    }

    /// The revision number of this PhyIdent
    pub const fn revision(&self) -> u8 {
        (self.1) as u8 & 0x0F
    }

    /// Determine whether `self` and `other` are the same model from
    /// the same manufacturer, i.e. that their `oui`s and
    /// `model_number`s are identical.
    pub const fn is_same_mode(&self, other: &Self) -> bool {
        self.oui() == other.oui() && self.model_number() == other.model_number()
    }
}

/// An IEEE 802.3 compatible PHY that can be managed
/// using the Media Independent Interface Management (MIIM)
/// Interface.
pub trait Miim {
    /// Read the MIIM register at `address`.
    ///
    /// This is a read as specified by Clause 22 of
    /// the standard.
    #[doc(alias = "clause22_read")]
    #[doc(alias = "clause 22")]
    fn read_raw(&mut self, address: RegisterAddress) -> u16;

    /// Write `value` to the MIIM register at `address`.
    ///
    /// This is a write as specified by Clause 22 of
    /// the standard.
    #[doc(alias = "clause22_write")]
    #[doc(alias = "clause 22")]
    fn write_raw(&mut self, address: RegisterAddress, value: u16);

    /// Read `Register` on this PHY.
    #[doc(alias = "clause 22")]
    fn read<R: Register>(&mut self) -> R {
        let raw = self.read_raw(R::ADDRESS);
        raw.into()
    }

    /// Write `Register` on this PHY.
    #[doc(alias = "clause 22")]
    fn write<R: Register>(&mut self, value: R) {
        let raw = value.into();
        self.write_raw(R::ADDRESS, raw);
    }

    /// Modify `Register` on this PHY.
    fn modify<R: Register, F>(&mut self, f: F)
    where
        F: FnOnce(&mut R),
    {
        let mut value = self.read();
        f(&mut value);
        self.write(value);
    }

    /// Check if the PHY is currently resetting
    fn is_resetting(&mut self) -> bool {
        self.read::<BasicControl>().reset()
    }

    /// Reset the PHY. Verify that the reset has completed by checking
    /// [`Self::is_resetting() == false`][Self::is_resetting] before continuing usage.
    fn reset(&mut self) {
        self.modify(|bcr: &mut BasicControl| {
            bcr.set_reset(true);
        });
    }

    /// Perform a reset and block until the reset has completed
    fn blocking_reset(&mut self) {
        self.reset();
        while self.is_resetting() {}
    }

    /// Read the basic status register for this PHY.
    fn status(&mut self) -> BasicStatus {
        self.read()
    }

    /// Check if the PHY reports its link as being up
    fn phy_link_up(&mut self) -> bool {
        self.status().link_status()
    }

    /// Read the PHY identifier for this PHY.
    ///
    /// Returns `None` if the PHY does not support extended capabilities, i.e.
    /// [`Self::status().extended_capabilities() == false`][Self::status]
    fn phy_ident(&mut self) -> Option<PhyIdent> {
        if self.status().extended_capabilities() {
            const PHY_IDENT_1: RegisterAddress = RegisterAddress::new(2).unwrap();
            const PHY_IDENT_2: RegisterAddress = RegisterAddress::new(3).unwrap();

            let msb = self.read_raw(PHY_IDENT_1);
            let lsb = self.read_raw(PHY_IDENT_2);
            Some(PhyIdent::new(msb, lsb))
        } else {
            None
        }
    }

    /// The best advertisement this PHY supports and restart autonegotation.
    ///
    /// "Best", in this case, means largest amount of supported features
    fn set_best_autonegotation_advertisement(&mut self) {
        let status: BasicStatus = self.read();

        // Extended capabilities are required to configure
        // autonegotiation.
        if !status.extended_capabilities() {
            return;
        }

        // Extended status == 1000BASE-T able
        if status.extended_status() {
            let extended: ExtendedStatus = self.read();
            let _1000base_t_fd = extended._1000base_t_fd();
            let _1000base_t_hd = extended._1000base_t_hd();

            self.modify(|r: &mut LeaderFollowerControl| {
                r.set__1000base_t_fd(_1000base_t_fd);
                r.set__1000base_t_hd(_1000base_t_hd);
            });
        }

        // Ignore 100baset4
        let _100base_x_fd = status._100base_x_fd();
        let _100base_x_hd = status._100base_x_hd();
        let _10base_t_fd = status._10base_t_fd();
        let _10base_t_hd = status._10base_t_hd();

        self.modify(|r: &mut AutonegotiationAdvertisement| {
            let mut tech_ability = r.technology_ability();
            tech_ability.set__100base_tx_fd(_100base_x_fd);
            tech_ability.set__100base_tx_hd(_100base_x_hd);
            tech_ability.set__10base_t_fd(_10base_t_fd);
            tech_ability.set__10base_t_hd(_10base_t_hd);
            r.set_technology_ability(tech_ability);
        });

        self.modify(|bcr: &mut BasicControl| {
            bcr.set_link_config(LinkConfig::Autonegotiate { restart: true });
        })
    }

    /// Get the current link state of this PHY.
    ///
    /// If you wish to perform the process of determining the link
    /// state of a spec-compilant PHY by other means, check out
    /// [`GetLinkStateProcess::start`] and [`GetLinkStateProcess::oneshot`].
    fn get_link_state(&mut self) -> Result<LinkState, LinkStateError> {
        use core::ops::ControlFlow;

        let process = match GetLinkStateProcess::start(self.read(), self.read()) {
            ControlFlow::Continue(c) => c,
            ControlFlow::Break(res) => return res,
        };

        let process = match process.next(self.read(), self.read(), self.read()) {
            ControlFlow::Continue(c) => c,
            ControlFlow::Break(res) => return res,
        };

        process.next(self.read(), self.read())
    }

    /// Set the autonegotiation advertisement and restart the autonegotiation
    /// process
    ///
    /// This is a no-op if the PHY does not support extended capabilities, i.e.
    /// [`Self::status().extended_capabilities() == false`][Self::status]
    fn set_autonegotiation_advertisement(&mut self, ad: AutonegotiationAdvertisement) {
        let status = self.status();
        if !status.extended_capabilities() {
            return;
        }

        self.write(ad);

        self.modify(|bcr: &mut BasicControl| {
            bcr.set_link_config(LinkConfig::Autonegotiate { restart: true });
        })
    }

    /// Read an MMD register
    ///
    /// This is a read as specified by Clause 45 of
    /// the standard. The default implementation uses
    /// Clause 22 reads (see [`Mmd::read`]) to perform
    /// this operation.
    #[doc(alias = "clause45_read")]
    #[doc(alias = "clause 45")]
    fn mmd_read(&mut self, device_address: bilge::prelude::u5, reg_address: u16) -> u16
    where
        Self: Sized,
    {
        Mmd::read(self, device_address, reg_address)
    }

    /// Write an MMD register
    ///
    /// This is a write as specified by Clause 45 of
    /// the standard. The default implementation uses
    /// Clause 22 writes (see [`Mmd::write`]) to perform
    /// this operation.
    #[doc(alias = "clause45_write")]
    #[doc(alias = "clause 45")]
    fn mmd_write(&mut self, device_address: bilge::prelude::u5, reg_address: u16, reg_value: u16)
    where
        Self: Sized,
    {
        Mmd::write(self, device_address, reg_address, reg_value)
    }
}

#[cfg(test)]
mod link_ordering {
    use crate::{registers::Duplex, LinkSpeed, LinkState, Miim, RegisterAddress};

    #[test]
    fn gig_greatest() {
        assert!(LinkSpeed::Mbps1000 > LinkSpeed::Mbps100);
        assert!(LinkSpeed::Mbps1000 > LinkSpeed::Mbps10);
    }

    #[test]
    fn _100_bigger_than_10() {
        assert!(LinkSpeed::Mbps100 > LinkSpeed::Mbps10);
    }

    #[test]
    fn duplex_ordering() {
        assert!(Duplex::Full > Duplex::Half);
    }

    #[test]
    fn link_state_ordering() {
        assert!(
            LinkState {
                speed: LinkSpeed::Mbps1000,
                duplex: Duplex::Full
            } > LinkState {
                speed: LinkSpeed::Mbps1000,
                duplex: Duplex::Half
            }
        );

        assert!(
            LinkState {
                speed: LinkSpeed::Mbps1000,
                duplex: Duplex::Full
            } > LinkState {
                speed: LinkSpeed::Mbps100,
                duplex: Duplex::Full
            }
        )
    }

    struct MockPhy {
        registers: [u16; 16],
    }

    impl Miim for MockPhy {
        fn read_raw(&mut self, address: RegisterAddress) -> u16 {
            self.registers[address.get() as usize]
        }

        fn write_raw(&mut self, address: RegisterAddress, value: u16) {
            self.registers[address.get() as usize] = value;
        }
    }

    const GIGABIT_GIGABIT_PARTNER: MockPhy = MockPhy {
        #[rustfmt::skip]
        registers: [
            0x1000, 0x79ad, 0x001c, 0xc800, 0x0de1, 0xc1e1, 0x006d, 0x2001,
            0x6001, 0x0200, 0x3800, 0x0000, 0x0000, 0x0000, 0x0000, 0x2000,
        ],
    };

    const GIGABIT_100M_PARTNER: MockPhy = MockPhy {
        #[rustfmt::skip]
        registers: [
            0x1040, 0x79ad, 0x001c, 0xc800, 0x0de1, 0x51e1, 0x0065, 0x2001,
            0x0000, 0x0200, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x2000,
        ],
    };

    const GIGABIT_10M_PARTNER: MockPhy = MockPhy {
        #[rustfmt::skip]
        registers: [
        0x1000, 0x79ad, 0x001c, 0xc800, 0x01e1, 0x4061, 0x0067, 0x2801,
        0x0000, 0x0200, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x2000,
        ],
    };

    const LINK_NOT_UP: MockPhy = MockPhy {
        #[rustfmt::skip]
        registers: [
            0x1000, 0x7989, 0x001c, 0xc800, 0x0de1, 0x0000, 0x0064, 2801,
            0x0000, 0x0200, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 2000,
        ],
    };

    const AUTONEG_INCOMPLETE: MockPhy = MockPhy {
        #[rustfmt::skip]
        registers: [
            // Same as LINK_NOT_UP, but with link status bit set
            0x1000, 0x7989 | (1 << 2), 0x001c, 0xc800, 0x0de1, 0x0000, 0x0064, 2801,
            0x0000, 0x0200, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 2000,
        ],
    };

    #[test]
    fn link_not_up() {
        let mut phy = LINK_NOT_UP;

        let state = phy.get_link_state();

        assert_eq!(state, Err(crate::LinkStateError::NoLink))
    }

    #[test]
    fn autoneg_incomplete() {
        let mut phy = AUTONEG_INCOMPLETE;

        let state = phy.get_link_state();

        assert_eq!(
            state,
            Err(crate::LinkStateError::AutonegotiationNotCompleted)
        )
    }

    #[test]
    fn link_state_10fd() {
        let mut phy = GIGABIT_10M_PARTNER;

        let state = phy.get_link_state().unwrap();

        assert_eq!(
            state,
            LinkState {
                speed: crate::LinkSpeed::Mbps10,
                duplex: crate::registers::Duplex::Full
            }
        )
    }

    #[test]
    fn link_state_100fd() {
        let mut phy = GIGABIT_100M_PARTNER;

        let state = phy.get_link_state().unwrap();

        assert_eq!(
            state,
            LinkState {
                speed: crate::LinkSpeed::Mbps100,
                duplex: crate::registers::Duplex::Full
            }
        )
    }

    #[test]
    fn link_state_1gfd() {
        let mut phy = GIGABIT_GIGABIT_PARTNER;

        let state = phy.get_link_state().unwrap();

        assert_eq!(
            state,
            LinkState {
                speed: crate::LinkSpeed::Mbps1000,
                duplex: crate::registers::Duplex::Full
            }
        )
    }
}
