//! A bare phy that does not have any compile-time configurations
//! assigned to it.

use crate::{AutoNegotiationAdvertisement, Miim, Pause, Phy};

/// A base phy
pub struct BarePhy<MIIM>
where
    MIIM: Miim,
{
    phy_address: u8,
    miim: MIIM,
    best_supported_advertisement: AutoNegotiationAdvertisement,
}

impl<MIIM> BarePhy<MIIM>
where
    MIIM: Miim,
{
    /// Create a new bare PHY with the given MIIM, at the given PHY address, using
    /// `pause` as the advertised pause mode.
    ///
    /// The PHY will calculate it's best supported advertisement on the fly from
    /// details acquired through `miim`.
    pub fn new(miim: MIIM, phy_address: u8, pause: Pause) -> Self {
        let mut me = Self {
            phy_address,
            miim,
            best_supported_advertisement: Default::default(),
        };

        let mut ana = me.status().best_autoneg_ad();
        ana.pause = pause;

        me.best_supported_advertisement = ana;
        me
    }

    /// Release the underlying MIIM
    pub fn release(self) -> MIIM {
        self.miim
    }
}

impl<MIIM> Phy<MIIM> for BarePhy<MIIM>
where
    MIIM: Miim,
{
    fn best_supported_advertisement(&self) -> AutoNegotiationAdvertisement {
        self.best_supported_advertisement
    }

    fn get_mii_mut(&mut self) -> &mut MIIM {
        &mut self.miim
    }

    fn get_miim(&self) -> &MIIM {
        &self.miim
    }

    fn get_phy_addr(&self) -> u8 {
        self.phy_address
    }
}
