// mod mbc0;
use std::fs::{metadata, File};
use std::io::prelude::*;
use std::str;
mod empty;
mod mbc0;
mod mbc1;
use empty::MbcEmpty;
use mbc0::Mbc0;
use mbc1::Mbc1;

pub trait Mbc {
    fn rb(&self, address: u16) -> u8;
    fn wb(&mut self, address: u16, value: u8);
}

pub struct Cartridge {
    mbc: Box<dyn Mbc>,
}

/// For now the cartridge is not inserted.
impl Cartridge {
    /// Initialize the cartridge by determining from the header what memory bank controller to use.
    /// It is possible that no cartridge is installed.
    pub fn new(cartridge_path: Option<&String>) -> Self {
        let mbc: Box<dyn Mbc> = match cartridge_path {
            Some(path) => {
                let data = Self::load_cartridge_data(path);
                Self::report_cartridge_header(&data);

                match &data[0x147] {
                    0x00 => Box::new(Mbc0::new(data)),
                    0x01..=0x03 => Box::new(Mbc1::new(data)),
                    m => panic!("Tried to initialize non-supported MBC: {:x}", m),
                }
            }
            None => {
                println!("No cartridge provided.");
                Box::new(MbcEmpty::new())
            }
        };

        Self { mbc }
    }

    pub fn rb(&self, address: u16) -> u8 {
        self.mbc.rb(address)
    }

    /// Write to ROM.  This isn't actually a write, but the attempt to write will control
    /// on-cartridge ROM banking systems that will make a different bank of data available in the
    // top 16KB of ROM addressable space.
    pub fn wb(&mut self, address: u16, value: u8) {
        self.mbc.wb(address, value);
    }

    fn report_cartridge_header(data: &Vec<u8>) {
        let rom_size = 32 << &data[0x148];
        let bank_count = rom_size / 16;
        println!("Name: {}", str::from_utf8(&data[0x134..0x143]).unwrap());
        println!("MBC: {}", &data[0x147]);
        println!("ROM Size: {} KB ({} banks)", rom_size, bank_count);
    }

    /// Load a cartridge into memory.
    /// A vector is allocated because we don't know until runtime how large the cartridge is.
    fn load_cartridge_data(path: &String) -> Vec<u8> {
        let mut f = File::open(path).expect("No file found.");
        let metadata = metadata(path).expect("Unable to read metadata.");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer)
            .expect("Buffer overflow! was metaadta wrong?");
        buffer
    }
}
