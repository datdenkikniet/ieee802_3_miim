use std::num::ParseIntError;

use bilge::prelude::u5;
use ieee802_3_miim::{
    mdio::PhyAddress,
    registers::{auto_negotiation::AutonegotiationAdvertisement, BasicControl, ExtendedStatus},
    Miim, RegisterAddress,
};

use crate::ioctl_mdio::IoctlMdio;

mod ioctl_mdio {
    use ieee802_3_miim::RegisterAddress;
    use nix::{
        errno::Errno,
        libc::{
            close, ifreq, ioctl, socket, AF_INET, IFNAMSIZ, IPPROTO_IP, SIOCGMIIREG, SIOCSMIIREG,
            SOCK_DGRAM,
        },
    };

    #[repr(C)]
    struct MiiIoctlData {
        phy_id: u16,
        reg_num: u16,
        val_in: u16,
        val_out: u16,
    }

    pub struct IoctlMdio {
        fd: i32,
        ifr_name: [i8; IFNAMSIZ],
        phy_address: u16,
    }

    impl IoctlMdio {
        pub fn new(interface: &str, phy_address: u16) -> Self {
            let socket = unsafe { socket(AF_INET, SOCK_DGRAM, IPPROTO_IP) };

            if socket < 0 {
                panic!("Opening socket failed: {}", Errno::from_raw(socket.abs()));
            }

            let mut if_name = [0i8; _];
            let len = if_name.len();

            if_name
                .iter_mut()
                .zip(interface.as_bytes().iter())
                // Probably needs a trailing zero?
                .take(len - 1)
                .for_each(|(d, s)| {
                    *d = *s as i8;
                });

            Self {
                fd: socket,
                ifr_name: if_name,
                phy_address,
            }
        }
    }

    impl Drop for IoctlMdio {
        fn drop(&mut self) {
            unsafe { close(self.fd) };
        }
    }

    impl IoctlMdio {
        fn op(&mut self, ioctl_req: u64, address: u16, if_write: u16) -> u16 {
            let mut req = ifreq {
                ifr_name: self.ifr_name,
                ifr_ifru: unsafe { std::mem::zeroed() },
            };

            let data_in: &mut MiiIoctlData = unsafe { core::mem::transmute(&mut req.ifr_ifru) };

            data_in.phy_id = self.phy_address;
            data_in.reg_num = address;
            data_in.val_in = if_write;

            #[expect(
                dropping_references,
                reason = "we mustn't hold the reference across the IOCTL that modifies it"
            )]
            drop(data_in);

            let ioctl = unsafe { ioctl(self.fd, ioctl_req, &mut req) };

            if ioctl < 0 {
                panic!("IOCTL failed: {}", Errno::last());
            }

            let data_out: &mut MiiIoctlData = unsafe { core::mem::transmute(&mut req.ifr_ifru) };

            data_out.val_out
        }

        fn read_raw(&mut self, address: RegisterAddress) -> u16 {
            self.op(SIOCGMIIREG, address.get() as _, 0)
        }

        fn write_raw(&mut self, address: RegisterAddress, value: u16) {
            self.op(SIOCSMIIREG, address.get() as _, value);
        }

        fn mmd_read(&mut self, device_address: bilge::prelude::u5, reg_address: u16) -> u16 {
            // linux supports detection of MMD support through special PHY address,
            // PHY_ADDRESS = 0x8000 | (phy_address << 5) | dev_addr
            self.phy_address <<= 5;
            self.phy_address |= u16::from(device_address);
            self.phy_address |= 0x8000;

            let result = self.op(SIOCGMIIREG, reg_address, 0);

            self.phy_address >>= 5;
            self.phy_address &= 0x1f;

            result
        }

        fn mmd_write(
            &mut self,
            device_address: bilge::prelude::u5,
            reg_address: u16,
            reg_value: u16,
        ) {
            // linux supports detection of MMD support through special PHY address,
            // PHY_ADDRESS = 0x8000 | (phy_address << 5) | dev_addr
            self.phy_address <<= 5;
            self.phy_address |= u16::from(device_address);
            self.phy_address |= 0x8000;

            let _result = self.op(SIOCSMIIREG, reg_address, reg_value);

            self.phy_address >>= 5;
            self.phy_address &= 0x1f;
        }
    }

    impl ieee802_3_miim::Miim for &mut IoctlMdio {
        fn read_raw(&mut self, address: RegisterAddress) -> u16 {
            IoctlMdio::read_raw(self, address)
        }

        fn write_raw(&mut self, address: RegisterAddress, value: u16) {
            IoctlMdio::write_raw(self, address, value)
        }

        fn mmd_read(&mut self, device_address: bilge::prelude::u5, reg_address: u16) -> u16
        where
            Self: Sized,
        {
            IoctlMdio::mmd_read(self, device_address, reg_address)
        }

        fn mmd_write(
            &mut self,
            device_address: bilge::prelude::u5,
            reg_address: u16,
            reg_value: u16,
        ) where
            Self: Sized,
        {
            IoctlMdio::mmd_write(self, device_address, reg_address, reg_value);
        }
    }

    impl ieee802_3_miim::Miim for IoctlMdio {
        fn read_raw(&mut self, address: RegisterAddress) -> u16 {
            IoctlMdio::read_raw(self, address)
        }

        fn write_raw(&mut self, address: RegisterAddress, value: u16) {
            IoctlMdio::write_raw(self, address, value)
        }

        fn mmd_read(&mut self, device_address: bilge::prelude::u5, reg_address: u16) -> u16
        where
            Self: Sized,
        {
            IoctlMdio::mmd_read(self, device_address, reg_address)
        }

        fn mmd_write(
            &mut self,
            device_address: bilge::prelude::u5,
            reg_address: u16,
            reg_value: u16,
        ) where
            Self: Sized,
        {
            IoctlMdio::mmd_write(self, device_address, reg_address, reg_value);
        }
    }
}

enum MdioLocation {
    Clause22 { register: RegisterAddress },
    Clause45 { device_address: u5, register: u16 },
}

fn decimal_or_hex(str: &str) -> Result<u8, ParseIntError> {
    if let Some(suffix) = str.strip_prefix("0x") {
        u8::from_str_radix(suffix, 16)
    } else {
        u8::from_str_radix(str, 10)
    }
}

fn decimal_or_hex_u16(str: &str) -> Result<u16, ParseIntError> {
    if let Some(suffix) = str.strip_prefix("0x") {
        u16::from_str_radix(suffix, 16)
    } else {
        u16::from_str_radix(str, 10)
    }
}

impl MdioLocation {
    pub fn parse(input: &str) -> (String, PhyAddress, Option<Self>) {
        let split: Vec<_> = input.split("/").collect();

        if let Some([interface, phy_addr, reg]) = split.as_array() {
            let (phy_addr, location) = if let Some((phy_addr, dev_addr)) = phy_addr.split_once(':')
            {
                let reg = decimal_or_hex(reg).expect("invalid register address");
                let reg = u16::try_from(reg).expect("invalid register address");

                let device_address = decimal_or_hex(dev_addr).expect("invalid device address");
                let device_address = u5::try_new(device_address).expect("invalid device address");

                let phy_addr = decimal_or_hex(phy_addr).expect("invalid PHY address");
                let phy_addr = PhyAddress::new(phy_addr).expect("invalid PHY address");

                (
                    phy_addr,
                    Self::Clause45 {
                        device_address,
                        register: reg,
                    },
                )
            } else {
                let reg = decimal_or_hex(reg).expect("invalid register address");
                let reg = RegisterAddress::new(reg).expect("invalid register address");

                let phy_addr = decimal_or_hex(phy_addr).expect("invalid PHY address");
                let phy_addr = PhyAddress::new(phy_addr).expect("invalid PHY address");

                (phy_addr, Self::Clause22 { register: reg })
            };

            (interface.to_string(), phy_addr, Some(location))
        } else if let Some([interface, phy_addr]) = split.as_array() {
            let phy_addr = decimal_or_hex(phy_addr).expect("invalid PHY address");
            let phy_addr = PhyAddress::new(phy_addr).expect("invalid PHY address");

            (interface.to_string(), phy_addr, None)
        } else {
            panic!("Invalid address");
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);

    let Some(command) = args.next() else {
        todo!("Print help")
    };

    let Some(reg) = args.next() else {
        todo!("Print help")
    };

    let (iface, phy_addr, reg) = MdioLocation::parse(&reg);

    let mut mdio = IoctlMdio::new(&iface, phy_addr.get() as _);

    if command == "link_status" {
        let state = mdio.get_link_state();
        println!("Link state: {state:?}");
    } else if command == "read" {
        let reg = reg.expect("read requires register");

        let value = match reg {
            MdioLocation::Clause22 { register, .. } => mdio.read_raw(register),
            MdioLocation::Clause45 {
                device_address,
                register,
            } => mdio.mmd_read(device_address, register),
        };

        println!("0x{:04X}", value);
    } else if command == "write" {
        let reg = reg.expect("write requires register");

        let value: u16 = decimal_or_hex_u16(&args.next().expect("write requires value"))
            .expect("valid value to write");

        match reg {
            MdioLocation::Clause22 { register, .. } => mdio.write_raw(register, value),
            MdioLocation::Clause45 {
                device_address,
                register,
            } => mdio.mmd_write(device_address, register, value),
        };

        println!("Wrote value");
    } else if command == "print" {
        let fmt_print = if let Some(reg) = reg {
            match reg {
                MdioLocation::Clause22 { register } => fmt_clause22(&mut mdio, register),
                MdioLocation::Clause45 { .. } => todo!(),
            }
        } else {
            let link_state = match mdio.get_link_state() {
                Ok(v) => format!("{v:?}"),
                Err(e) => format!("{e:?}"),
            };

            let ad: AutonegotiationAdvertisement = mdio.read();
            let ad = ad.technology_ability();

            [
                ("Link state", link_state),
                ("Advertised capabilities", format!("{ad:?}")),
            ]
            .to_vec()
        };

        let width = fmt_print.iter().map(|(v, _)| v.len()).max().unwrap_or(0) + 1;

        for (name, value) in fmt_print {
            println!("{name:<width$} {value}");
        }
    } else {
        panic!("Unknown command {command}")
    }
}

fn fmt_clause22<M: Miim>(mut m: M, register: RegisterAddress) -> Vec<(&'static str, String)> {
    match register.get() {
        0 => {
            let control: BasicControl = m.read();
            let config = control.get_link_config();

            let on_off = |value| {
                if value {
                    "on".to_string()
                } else {
                    "off".to_string()
                }
            };

            [
                ("Link config", format!("{config:?}")),
                ("Reset", on_off(control.reset())),
                ("Loopback", on_off(control.loopback())),
                ("Powered down", control.power_down().to_string()),
                ("Isolation", on_off(control.isolate())),
                ("Collision test", on_off(control.collision_test())),
            ]
            .to_vec()
        }
        1 => {
            let status = m.status();

            let link_status = if status.link_status() {
                format!("up")
            } else {
                format!("Down")
            };

            [
                (
                    "Extended capabilities",
                    status.extended_capabilities().to_string(),
                ),
                ("Jabber detected", status.jabber_detect().to_string()),
                ("Link status", link_status),
                (
                    "Auto-negotiation able",
                    status.autonegotiate_able().to_string(),
                ),
                ("Remote fault", status.remote_fault().to_string()),
                (
                    "Auto-negotiation complete",
                    status.autonegotiation_complete().to_string(),
                ),
                (
                    "MF preamble suppression",
                    status.mf_preamble_suppression().to_string(),
                ),
                (
                    "Unidirectional able",
                    status.unidirectional_ability().to_string(),
                ),
                (
                    "Extended status available",
                    status.extended_status().to_string(),
                ),
                ("100BASE-T2 HD capable", status._100base_t2_hd().to_string()),
                ("100BASE-T2 FD capable", status._100base_t2_fd().to_string()),
                ("10BASE-T HD capable", status._10base_t_hd().to_string()),
                ("10BASE-T FD capable", status._10base_t_fd().to_string()),
                ("100BASE-X HD capable", status._100base_x_hd().to_string()),
                ("100BASE-X HD capable", status._100base_x_fd().to_string()),
            ]
            .to_vec()
        }
        15 => {
            let status = m.status();

            if status.extended_status() {
                let extended_status: ExtendedStatus = m.read();

                [
                    (
                        "1000BASE-T HD capable",
                        extended_status._1000base_t_hd().to_string(),
                    ),
                    (
                        "1000BASE-T FD capable",
                        extended_status._1000base_t_fd().to_string(),
                    ),
                    (
                        "1000BASE-X HD capable",
                        extended_status._1000base_x_hd().to_string(),
                    ),
                    (
                        "1000BASE-X FD capable",
                        extended_status._1000base_x_fd().to_string(),
                    ),
                ]
                .to_vec()
            } else {
                panic!("PHY does support extended status");
            }
        }
        _ => {
            let value = m.read_raw(register);
            println!("Raw value: 0x{value:04X}");
            return Vec::new();
        }
    }
}
