use std::fs::File;
use std::io::prelude::*;

pub struct Cartridge {
    data: [u8; 0x8000],
}

/// For now the cartridge is not inserted.
impl Cartridge {
    pub fn new(cartridge_path: Option<&String>) -> Self {
        // Either load the cartridge or return blank data.
        let data = match cartridge_path {
            Some(path) => Self::load_cartridge_data(path),
            None => [0xFF; 0x8000], // No cartridge, returns 0xFF
        };

        Self { data }
    }

    pub fn rb(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    /// Write to ROM.  This isn't actually a write, but the attempt to write will control
    /// on-cartridge ROM banking systems that will make a different bank of data available in the
    // top 16KB of ROM addressable space.
    pub fn wb(&self, address: u16, value: u8) {
        // TODO: handle banking. For now we just swallow the writes.
        println!(
            "Tried to write to cartridge: {:#06x} {:#04x}",
            address, value
        )
    }

    /// Load a cartridge into memory.
    /// TODO: support cartridges of different sizes using banking. It would return a vector.
    /// A banking mechanism (register based?) would decide which slice of that vector to expose.
    fn load_cartridge_data(path: &String) -> [u8; 0x8000] {
        let mut f = File::open(path).unwrap();
        let mut buffer = [0; 0x8000];
        f.read(&mut buffer[..]).unwrap();
        buffer
    }
}
