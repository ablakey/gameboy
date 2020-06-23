use pretty_hex::*;
use std::fs::File;
use std::io;
use std::io::prelude::*;

pub struct GamePak {
    buffer: Vec<u8>,
}

impl GamePak {
    /// Load a GamePak from a DMG-01 ROM file.
    /// This will not include any save data, which needs to be optionally loaded separately.
    /// A GamePak is typically 32KB but some games have more (up to 1MB even) and use memory bank
    /// switching to handle this. To emulate, we load the entire ROM in memory, and expose a slice
    // of it as needed.
    pub fn load_from_rom(path: &String) -> io::Result<Self> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut f = File::open(path)?;

        f.read_to_end(&mut buffer)?;

        let s: Self = Self { buffer };

        Ok(s)
    }
}

/// Debug Implementation.
impl GamePak {
    pub fn print_debug(&self) {
        println!("{} KB", self.buffer.len() / 1024);
        println!("{}", self.dump_loaded_rom());
    }

    pub fn dump_loaded_rom(&self) -> String {
        format!(
            "{:?}",
            self.buffer[0x0..0x100]
                .to_vec()
                .iter()
                .map(|&f| f as u8)
                .collect::<Vec<u8>>()
                .hex_dump()
        )
    }
}
