//! Implementations of MII for existing PHYs

#[cfg(feature = "lan87xxa")]
pub mod lan87xxa;

#[cfg(feature = "kzs8081r")]
pub mod ksz8081r;

pub mod bare;
