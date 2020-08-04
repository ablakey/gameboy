use std::fs::File;
use std::io;
use std::io::prelude::*;

const BOOT_ROM_PATH: &'static str = "data/dmg_rom.bin";

pub struct BootRom {
    data: [u8; 0x100],
    pub is_enabled: bool,
}

impl BootRom {
    pub fn new() -> Self {
        let data = Self::load_boot_rom().unwrap();
        Self {
            data,
            is_enabled: true,
        }
    }

    /// Load the boot loader ROM from file.
    /// This is a 256byte ROM referencable at 0x00 - 0xFF, containing the logic for validating
    /// that the cartridge is legitimate, scolling the Nintendo logo and playing the chime.
    pub fn load_boot_rom() -> io::Result<[u8; 0x100]> {
        let mut f = File::open(BOOT_ROM_PATH)?;
        let mut buffer = [0; 0x100];
        f.read(&mut buffer[..])?;
        Ok(buffer)
    }

    pub fn rb(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }
}
