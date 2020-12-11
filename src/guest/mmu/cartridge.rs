use std::fs::{metadata, File};
use std::io::prelude::*;
use std::str;

pub struct Cartridge {
    data: Option<Vec<u8>>,
}

/// For now the cartridge is not inserted.
impl Cartridge {
    pub fn new(cartridge_path: Option<&String>) -> Self {
        // Either load the cartridge or return blank data.
        let data = match cartridge_path {
            Some(path) => Some(Self::load_cartridge_data(path)),
            None => None, // No cartridge, returns 0xFF
        };

        match &data {
            Some(data) => Self::report_cartridge_header(data),
            None => println!("No cartridge provided."),
        }

        Self { data }
    }

    pub fn rb(&self, address: u16) -> u8 {
        match &self.data {
            Some(data) => *data
                .get(address as usize)
                .expect("Tried to get data out of bounds!"),
            None => 0xFF,
        }
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

    fn report_cartridge_header(data: &Vec<u8>) {
        let rom_size = 32 << &data[0x148];
        let bank_count = rom_size / 16;
        println!("Name: {}", str::from_utf8(&data[0x134..0x143]).unwrap());
        println!("MBC: {}", &data[0x147]);
        println!("ROM Size: {} KB ({} banks)", rom_size, bank_count);
    }

    /// Load a cartridge into memory.
    /// TODO: support cartridges of different sizes using banking. It would return a vector.
    /// A banking mechanism (register based?) would decide which slice of that vector to expose.
    fn load_cartridge_data(path: &String) -> Vec<u8> {
        let mut f = File::open(path).expect("No file found.");
        let metadata = metadata(path).expect("Unable to read metadata.");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer)
            .expect("Buffer overflow! was metaadta wrong?");
        buffer
    }
}
