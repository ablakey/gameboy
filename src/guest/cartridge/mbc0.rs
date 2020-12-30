use super::Mbc;

pub struct Mbc0 {
    data: Vec<u8>,
}

impl Mbc0 {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl Mbc for Mbc0 {
    fn rb(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    fn wb(&mut self, _address: u16, _value: u8) {}
}
