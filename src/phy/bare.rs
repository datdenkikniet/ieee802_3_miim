//! A bare phy that does not have any compile-time configurations
//! assigned to it.

use crate::{AutoNegotiationAdvertisement, Miim, Pause, Phy};

/// A base phy
#[derive(Debug)]
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

    /// Change the PHY address
    pub fn set_phy_addr(&mut self, phy_address: u8) {
        self.phy_address = phy_address;
    }
}

impl<MIIM> Phy<MIIM> for BarePhy<MIIM>
where
    MIIM: Miim,
{
    fn best_supported_advertisement(&self) -> AutoNegotiationAdvertisement {
        self.best_supported_advertisement
    }

    fn get_miim(&mut self) -> &mut MIIM {
        &mut self.miim
    }

    fn get_phy_addr(&self) -> u8 {
        self.phy_address
    }
}

pub enum IdentPhyError {
    PhyIdentUnavailable,
    IncorrectPhyIdent,
}

macro_rules! into_phy {
    ($([$feat:literal, $phy:ident, $id:literal],)*) => {
        $(
            #[cfg(feature = $feat)]
            impl<MIIM: Miim> TryFrom<BarePhy<MIIM>> for super::$phy<MIIM> {
                type Error = IdentPhyError;

                fn try_from(mut value: BarePhy<MIIM>) -> Result<Self, Self::Error> {
                    let phy_ident = value.phy_ident().ok_or(IdentPhyError::PhyIdentUnavailable)?.raw_u32();

                    if phy_ident & 0xFFFFFFF0 == $id {
                        Ok(super::$phy::new(value.miim, value.phy_address))
                    } else {
                        Err(IdentPhyError::IncorrectPhyIdent)
                    }
                }
            }
        )*
    };
}

into_phy!(
    ["ksz8081r", KSZ8081R, 0x00221560],
    ["lan8720a", LAN8720A, 0x0007C0F0],
    ["lan8742a", LAN8742A, 0x0007C130],
);
