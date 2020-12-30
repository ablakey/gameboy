use super::Mbc;

pub struct Mbc0 {
    data: Vec<u8>,
}

impl Mbc0 {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

/// MBC 0 is a simple controller for cartridges with 16KB of ROM and no RAM. The one and only
/// memory bank is fully addressable so nothing fancy has to happen.
impl Mbc for Mbc0 {
    /// Read 0x000 - 0x7FFF directly.
    fn rb(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    fn wb(&mut self, _address: u16, _value: u8) {}
}
