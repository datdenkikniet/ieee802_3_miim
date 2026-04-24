# IEEE 802.3 Media Independent Interface Management

A crate traits for accessing the Media Independent Interface Management on IEEE 802.3 PHYs.

## What is Media Independent Interface Management?

The MIIM (Media Independent Interface Management) Interface is a part of the IEEE 802.3 standard used to manage the state of PHYs.

In it's most cut-down form, it provides basic configuration and status access. Extended features (implemented by most PHYs)
include autonegotiation configuration, custom on-chip register access through MMD, and extended status information.

## What is MDIO?

MDIO (Management Data Input/Output) is one of the standard-mandated method of accessing MIIM functions. Most PHYs support
this protocol for accessing their MIIM interface, so the crate provides a default PHY-ish implementation (`ieee802_3_miim::mdio::MdioPhy`)
that will let you access your PHYs over an MDIO bus. However: some PHYs (such as the Microchip KSZ8863) also provide access to
MIIM registers over other interfaces. Therefore, the MIIM functionality provided by this crate is not tightly coupled to access using MDIO.

## Using this crate

The core trait, `Miim`, provides the functionality most users want:  the speed of the PHY as reported by its registers (through
`Miim::get_link_state`), and link status. Since it is does not provide access by itself, you'll need an implementation of `Miim`
to actually communicate with your PHY.

For most use-cases, the `ieee802_3_miim::mdio::MdioPhy` gives you what you need: it lets you instantiate
a generic `Miim` based on an MDIO implementation (usually provided by your HAL) and a PHY address.

### Non-generic PHYs

Other PHY implementations (`ieee802_3_miim::phy`) contain functionality specialized for specific PHYs:
* Interrupt configuration through vendor-specific MIIM registers (`lan87xxa`, `ksz8081r`)
* Custom PHY initialization sequences (`lan87xxa`)
* Custom speed detection implementation for fixed-function PHYs (`lan8770`)

However, these PHYs _also_ implement `Miim`, so if your code is generic over an `impl Miim`, you
can swap them how you see fit.

More custom PHY implementations are always welcome :)

# License
This project is licensed under the MIT license.

See `LICENSE` for more information.