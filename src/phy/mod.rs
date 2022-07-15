//! Implementations of MIIM for existing PHYs

#[cfg(any(feature = "lan8720a", feature = "lan8742a"))]
pub mod lan87xxa;

#[cfg(feature = "kzs8081r")]
pub mod ksz8081r;

pub mod bare;
