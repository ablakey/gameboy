use super::Mbc;

pub struct Mbc1 {
    data: Vec<u8>,
}

impl Mbc1 {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl Mbc for Mbc1 {
    fn rb(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    fn wb(&self, _address: u16, _value: u8) {}
}
