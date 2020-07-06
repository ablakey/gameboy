use std::fs::File;
use std::io;
use std::io::prelude::*;

type BootLoader = [u8; 0x100];
type Memory = [u8; 0x10000]; // 64KB of DMG-01 memory.

pub struct MMU {
    memory: Memory,
}

/// MMU wrapping all I/O for memory.
/// DMG-01 has a 16-bit addressing bus and therefore can address 64KB of memory.
/// This includes 32 KB
///
/// I played with the idea of making this a more abstract wrapper of data by implementing Index
/// so that we could address memory that would actually be found in different locations. For example
/// instead of writing the boot_loader to the first 256 bytes, any access to mmu[0x00] to mmu[0xFF]
/// Would first look at mmu.boot_loader_active and decide whether to address mmu.boot_loader.
/// I decided this would be very slow, as every memory lookup would have a conditional test first.
/// It's likely better to just have a concrete 64KB of memory and copy data into it when handling
/// ROM banking and the boot loader.
///
/// It's probably useful for devel mode to override Index and test that we never try to write
/// to ROM addresses.
impl MMU {
    const BOOT_ROM_PATH: &'static str = "data/dmg_rom.bin";

    /// Initialize the MMU by loading the boot_rom into the first 256 addressable bytes.
    pub fn new() -> Self {
        let boot_loader = Self::load_boot_rom().unwrap();
        let mut memory = [0; 0x10000];
        memory[0..0x100].clone_from_slice(&boot_loader);

        Self { memory }
    }

    /// Read a byte from address.
    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    /// Read a word from address.
    /// DMG-01 is little endian so the least-significant byte is read first.
    pub fn read_word(&self, address: u16) -> u16 {
        let lsb = self.read_byte(address) as u16;
        let msb = self.read_byte(address + 1) as u16;
        (msb << 8) | lsb
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    /// Load the boot loader ROM from file.
    /// This is a 256byte ROM referencable at 0x00 - 0xFF, containing the logic for validating
    /// that the cartridge is legitimate, scolling the Nintendo logo and playing the chime.
    fn load_boot_rom() -> io::Result<BootLoader> {
        let mut f = File::open(Self::BOOT_ROM_PATH)?;
        let mut buffer = [0; 0x100];
        f.read(&mut buffer[..])?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_reads_a_word() {
        let mut mmu = MMU::new();
        mmu.memory[0] = 0xFF;
        mmu.memory[1] = 0x11;
        let word = mmu.read_word(0x00);
        assert_eq!(word, 0x11FF);
    }
}
