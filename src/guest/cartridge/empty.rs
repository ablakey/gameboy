use super::Mbc;

pub struct MbcEmpty {}

impl MbcEmpty {
    pub fn new() -> Self {
        Self {}
    }
}

impl Mbc for MbcEmpty {
    fn rb(&self, _address: u16) -> u8 {
        0xFF
    }

    fn wb(&self, _address: u16, _value: u8) {}
}
