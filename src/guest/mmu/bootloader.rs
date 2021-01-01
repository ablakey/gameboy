use std::fs::File;
use std::io;
use std::io::prelude::*;

const BOOT_ROM_PATH: &'static str = "data/dmg_rom.bin";

/// The values applied to the final state of the MMU once the boot rom has been run.

pub const BOOTROM_MMU_VALUES: [(u16, u8); 31] = [
    (0xFF05, 0),
    (0xFF06, 0),
    (0xFF07, 0),
    (0xFF10, 0x80),
    (0xFF11, 0xBF),
    (0xFF12, 0xF3),
    (0xFF14, 0xBF),
    (0xFF16, 0x3F),
    (0xFF16, 0x3F),
    (0xFF17, 0),
    (0xFF19, 0xBF),
    (0xFF1A, 0x7F),
    (0xFF1B, 0xFF),
    (0xFF1C, 0x9F),
    (0xFF1E, 0xFF),
    (0xFF20, 0xFF),
    (0xFF21, 0),
    (0xFF22, 0),
    (0xFF23, 0xBF),
    (0xFF24, 0x77),
    (0xFF25, 0xF3),
    (0xFF26, 0xF1),
    (0xFF40, 0x91),
    (0xFF42, 0),
    (0xFF43, 0),
    (0xFF45, 0),
    (0xFF47, 0xFC),
    (0xFF48, 0xFF),
    (0xFF49, 0xFF),
    (0xFF4A, 0),
    (0xFF4B, 0),
];

pub struct BootLoader {
    data: [u8; 0x100],
    pub is_enabled: bool,
}

impl BootLoader {
    pub fn new(use_bootrom: bool) -> Self {
        if use_bootrom {
            Self {
                data: Self::load_boot_rom().unwrap(),
                is_enabled: true,
            }
        } else {
            Self {
                data: [0; 0x100],
                is_enabled: false,
            }
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
