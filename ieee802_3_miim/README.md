# IEEE 802.3 Media Independent Interface Management

A crate traits for accessing the Media Independent Interface Management on IEEE 802.3 PHYs.

## What is Media Independent Interface Management?

The MIIM (Media Independent Interface Management) Interface is a part of the IEEE 802.3 standard used to manage the state of PHYs.

In it's most cut-down form, it provides basic configuration and status access. Extended features (implemented by most PHYs)
include autonegotiation configuration, custom on-chip register access through MMD, and extended status information.

## What is MDIO?

MDIO (Management Data Input/Output) is one of the standard-mandated method of accessing MIIM functions. Most PHYs support
this protocol for accessing their MIIM interface, so the crate provides a default PHY implementation (`MdioPhy`)
that will let you access your PHYs over an MDIO bus. However: some PHYs (such as the Microchip KSZ8863) also provide access to
MIIM registers over other interfaces. Therefore, the MIIM functionality provided by this crate is not tightly coupled to access using MDIO.

# Getting started

There are several ways to get started, depending on what your goal with the crate is.

## Application authors

The core trait, `Miim`, provides the functionality most users want: the speed of the PHY as reported by its registers (through
[`Miim::get_link_state`]), and link status. Since the trait is a trait, you'll need an implementation of [`Miim`]
to actually communicate with your PHY. This implementation is generally provided by your HAL, either as a direct implementation of
[`Miim`], or as an implementation of [`Mdio`], which can be used directly with an [`MdioPhy`].

If the HAL you are using does not support this crate directly, implementing the [`Miim`] trait on a wrapper type is likely a decent
alternative: the trait has few required functions, and provides a lot of functionality on top of them.

## HAL authors

HALs can provide the following:
* If your device supports MDIO, provide an implementation of the [`Mdio`] trait. All other functionality ([`Miim`], [`MdioPhy`]) will
  be available to your users based on it.
* If your device can provide access to MIIM registers in some other way (i.e. to only a single, specific PHY address on an MDIO bus), provide
  an implemetation of [`Miim`] directly.

## PHY driver authors

PHY drivers should, in general, aim to use an instance of [`Miim`] to perform their operations. See the example implemetations in the [`phy`]
module for how to use it.

Additionally, it is recommended that you implement [`Miim`] for your PHY itself, delegating to the [`Miim`] implementation that it owns: this way,
all functionality provided by the trait will be available to users of your driver, practically for free. Make sure to also re-export the trait
from your crate if you choose to do this.

# Non-generic PHYs

This crate also provides some PHY implementations (`ieee802_3_miim::phy`), containing specialized functionality for those PHYs:
* Interrupt configuration through vendor-specific MIIM registers (`lan87xxa`, `ksz8081r`)
* Custom PHY initialization sequences (`lan87xxa`)
* Custom speed detection implementation for fixed-function PHYs (`lan8770`)

However, these PHYs _also_ implement [`Miim`], so if your code is generic over an `impl Miim`, you
can swap them however you see fit.

More custom PHY implementations are always welcome :)

[`Mdio`]: crate::mdio::Mdio
[`phy`]: crate::phy
[`MdioPhy`]: crate::mdio::MdioPhy
[`Miim`]: crate::Miim
[`Miim::get_link_state`]: crate::Miim::get_link_state

# License
This project is licensed under the MIT license.

See `LICENSE` for more information.