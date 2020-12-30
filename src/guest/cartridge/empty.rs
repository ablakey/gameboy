use super::Mbc;

pub struct MbcEmpty {}

impl MbcEmpty {
    pub fn new() -> Self {
        Self {}
    }
}

impl Mbc for MbcEmpty {
    fn rb(&self, _address: u16) -> u8 {
        // When no cartridge is installed, all reads return 0xFF. All this really matters for
        // is providing the black block where the Nintendo logo should go.
        0xFF
    }

    fn wb(&mut self, _address: u16, _value: u8) {}
}
