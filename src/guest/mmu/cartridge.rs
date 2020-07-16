pub struct Cartridge {
    data: [u8; 0x8000],
}

/// For now the cartridge is not inserted.
impl Cartridge {
    pub fn new() -> Self {
        Self { data: [0; 0x8000] }
    }

    pub fn rb(&self, address: u16) -> u8 {
        self.data[address as usize]
    }
}
