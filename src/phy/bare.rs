//! A bare phy that does not have any compile-time configurations
//! assigned to it.

use crate::{AutoNegotiationAdvertisement, Mii, Pause, Phy};

/// A base phy
pub struct BarePhy<MII>
where
    MII: Mii,
{
    phy_address: u8,
    mii: MII,
    best_supported_advertisement: AutoNegotiationAdvertisement,
}

impl<MII> BarePhy<MII>
where
    MII: Mii,
{
    /// Create a new bare PHY with the given MII, at the given PHY address, using
    /// `pause` as the advertised pause mode.
    ///
    /// The PHY will calculate it's best supported advertisement on the fly from
    /// details acquired through `mii`.
    pub fn new(mii: MII, phy_address: u8, pause: Pause) -> Self {
        let mut me = Self {
            phy_address,
            mii,
            best_supported_advertisement: Default::default(),
        };

        let mut ana = me.status().best_autoneg_ad();
        ana.pause = pause;

        me.best_supported_advertisement = ana;
        me
    }

    /// Release the underlying MII
    pub fn release(self) -> MII {
        self.mii
    }
}

impl<MII> Phy<MII> for BarePhy<MII>
where
    MII: Mii,
{
    fn best_supported_advertisement(&self) -> AutoNegotiationAdvertisement {
        self.best_supported_advertisement
    }

    fn get_mii_mut(&mut self) -> &mut MII {
        &mut self.mii
    }

    fn get_mii(&self) -> &MII {
        &self.mii
    }

    fn get_phy_addr(&self) -> u8 {
        self.phy_address
    }
}
