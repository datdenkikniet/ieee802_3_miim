//! Implementations of MIIM for existing PHYs

#[cfg(any(feature = "lan8720a", feature = "lan8742a"))]
pub mod lan87xxa;
#[cfg(any(feature = "lan8720a", feature = "lan8742a"))]
pub use lan87xxa::{LAN8720A, LAN8742A};

#[cfg(feature = "ksz8081r")]
mod ksz8081r;
#[cfg(feature = "ksz8081r")]
pub use ksz8081r::KSZ8081R;

mod lan8770;

pub use lan8770::Lan8770;
