# IEEE 802.3 Media Independent Interface Management

A crate traits for accessing the Media Independent Interface Management on IEEE 802.3 PHYs.

## What is Media Independent Interface Management?

MIIM (Media Independent Interface Management) is a standard interface that is part of the IEEE 802.3 standard.

It is used to communicate with IEEE 802.3 PHYs. In it's most cut-down form, it provides basic configuration and status access. Extended features
include autonegotiation configuration, custom on-chip register access through MMD, and extended status information.

## PHY implementations
Several standard implementations are provided with the enabled-by-default `phy`, `lan8742a`, `lan8720a`, and `ksz8081r` features.

* `phy` exposes a type named `BarePhy`. This implementation assumes nothing about the PHY that is being communicated with, and determines almost all values at runtime. It should be possible to configure any IEEE 802.3 conformant PHY through this struct.
* `lan8742a` provides an implementation for the SMSC LAN8742a PHY.
* `lan8720a` provides an implementation for the SMSC LAN8720a PHY. Note that `Interrupt::WoL` is _not_ supported by this PHY, but it will be present if the `lan8742a` feature is also enabled.
* `ksz8081r` provides an implementation for the MicroChip KSZ8081R PHY