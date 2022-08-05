//! Implementations of MIIM for existing PHYs

use crate::{Miim, Phy};

#[cfg(any(feature = "lan8720a", feature = "lan8742a"))]
pub mod lan87xxa;
#[cfg(any(feature = "lan8720a", feature = "lan8742a"))]
pub use lan87xxa::{LAN8720A, LAN8742A};

#[cfg(feature = "kzs8081r")]
mod ksz8081r;
#[cfg(feature = "kzs8081r")]
pub use ksz8081r::KSZ8081R;

mod bare;
pub use bare::BarePhy;

/// Basic link speeds, supported by (almost all) PHYs
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhySpeed {
    /// 10BaseT - Half duplex
    HalfDuplexBase10T,
    /// 10BaseT - Full duplex
    FullDuplexBase10T,
    /// 100BaseTx - Half duplex
    HalfDuplexBase100Tx,
    /// 100BaseTx - Full duplex
    FullDuplexBase100Tx,
}

/// An "advanced link speed" enum that covers more than just the
/// basic ones described by the standard.

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum AdvancedPhySpeed {
    HalfDuplexBase10T,
    FullDuplexBase10T,
    HalfDuplexBase100Tx,
    FullDuplexBase100Tx,
    HalfDuplexBase1000T,
    FullDuplexBase1000T,
    HalfDuplexBase1000Tx,
    FullDuplexBase1000Tx,
}

impl From<PhySpeed> for AdvancedPhySpeed {
    fn from(s: PhySpeed) -> Self {
        match s {
            PhySpeed::HalfDuplexBase10T => Self::HalfDuplexBase10T,
            PhySpeed::FullDuplexBase10T => Self::FullDuplexBase10T,
            PhySpeed::HalfDuplexBase100Tx => Self::HalfDuplexBase100Tx,
            PhySpeed::FullDuplexBase100Tx => Self::FullDuplexBase100Tx,
        }
    }
}

/// A PHY that also supports determining the link speed and duplex mode
/// it is currently operating at.
pub trait PhyWithSpeed<MIIM: Miim>: Phy<MIIM> {
    /// Get the link speed at which this PHY is currently
    /// operating.
    fn get_link_speed(&mut self) -> Option<AdvancedPhySpeed>;
}
