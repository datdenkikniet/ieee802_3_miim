use crate::{
    registers::{BasicStatus, DuplexMode},
    LinkSpeed, LinkStateError, Miim, RegisterAddress,
};

/// A LAN8770 PHY.
#[derive(Debug, Clone, Copy)]
pub struct Lan8770<MIIM: Miim> {
    /// The MIIM instance of this PHY.
    pub miim: MIIM,
}

impl<MIIM: Miim> Lan8770<MIIM> {
    /// Create a new instance of this struct.
    pub fn new(miim: MIIM) -> Self {
        Self { miim }
    }
}

impl<MIIM: Miim> Miim for Lan8770<MIIM> {
    fn read_raw(&mut self, address: RegisterAddress) -> u16 {
        self.miim.read_raw(address)
    }

    fn write_raw(&mut self, address: RegisterAddress, value: u16) {
        self.miim.write_raw(address, value)
    }

    fn get_link_state(&mut self) -> Result<crate::LinkState, LinkStateError> {
        let basic_status: BasicStatus = self.read();
        if !basic_status.link_status() {
            Err(LinkStateError::NoLink)
        } else {
            Ok(crate::LinkState {
                speed: LinkSpeed::Mbps100,
                duplex: DuplexMode::Full,
            })
        }
    }
}
