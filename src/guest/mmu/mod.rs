mod registers;
use std::fs::File;
use std::io;
use std::io::prelude::*;
type BootLoader = [u8; 0x100];
type SRAM = [u8; 0x2000]; // 8KB of DMG-01 memory.

const STACK_TOP: u16 = 0xDFFF;

// TODO explain (that MMU has memory, registers, io regsiters (TBD) and other state)
pub struct MMU {
    sram: SRAM,
    pub pc: u16,
    pub sp: u16,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    f: u8,
}

// TODO explain
impl MMU {
    const BOOT_ROM_PATH: &'static str = "data/dmg_rom.bin";

    /// Initialize the MMU by loading the boot_rom into the first 256 addressable bytes.
    pub fn new() -> Self {
        let boot_loader = Self::load_boot_rom().unwrap();
        let mut sram = [0; 0x2000];
        sram[0..0x100].clone_from_slice(&boot_loader);

        Self {
            sram,
            pc: 0,
            sp: STACK_TOP + 1, // Stack increases downwards. Start one word above.
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            f: 0,
        }
    }

    /// Read a byte from address.
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xC000..=0xDFFF => self.sram[(address - 0xC000) as usize],
            _ => panic!("Tried to read from {:#x} which is not mapped.", address),
        }
    }

    /// Read a word from address.
    /// DMG-01 is little endian so the least-significant byte is read first.
    pub fn read_word(&self, address: u16) -> u16 {
        let lsb = self.read_byte(address) as u16;
        let msb = self.read_byte(address + 1) as u16;
        (msb << 8) | lsb
    }

    /// Write an 8-bit value to an address.
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xC000..=0xDFFF => self.sram[(address - 0xC000) as usize] = value,
            _ => panic!("Tried to write to {:#x} which is not mapped.", address),
        }
    }

    /// Write a 16-bit value to an address and the immediate address after.
    /// DMG-01 is little endian so the least-significant byte is written first.
    pub fn write_word(&mut self, address: u16, value: u16) {
        self.write(address, (value & 0xFF) as u8); // Mask only the LSB.
        self.write(address + 1, (value >> 8) as u8); // bit-shift until we have only the MSB.
    }

    /// Get the next byte and advance the program counter by 1.
    pub fn get_next_byte(&mut self) -> u8 {
        let byte = self.read_byte(self.pc);
        self.pc += 1;
        byte
    }

    /// Get the next byte as a two's complement signed integer and advance the program counter by 1.
    pub fn get_signed_byte(&mut self) -> i8 {
        self.get_next_byte() as i8
    }

    /// Get the next word in memory and advance the program counter by 2.
    pub fn get_next_word(&mut self) -> u16 {
        let word = self.read_word(self.pc);
        self.pc += 2;
        word
    }

    /// Load the boot loader ROM from file.
    /// This is a 256byte ROM referencable at 0x00 - 0xFF, containing the logic for validating
    /// that the cartridge is legitimate, scolling the Nintendo logo and playing the chime.
    pub fn load_boot_rom() -> io::Result<BootLoader> {
        let mut f = File::open(Self::BOOT_ROM_PATH)?;
        let mut buffer = [0; 0x100];
        f.read(&mut buffer[..])?;
        Ok(buffer)
    }

    /// Push a word (an address of the an instruction) to the stack.
    /// Stack decrements by one first (it grows downward in address space at the top of low RAM).
    pub fn push_stack(&mut self, address: u16) {
        self.sp -= 2;
        println!("{:x}", self.sp);
        self.write_word(self.sp, address);
    }

    /// Pop a word off the stack.
    /// It will go into a register.
    pub fn pop_stack(&mut self) -> u16 {
        let address = self.read_word(self.sp);
        self.sp += 2;
        address
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_word() {
        let mut mmu = MMU::new();
        mmu.sram[0] = 0xFF;
        mmu.sram[1] = 0x11;
        let word = mmu.read_word(0xC000);
        assert_eq!(word, 0x11FF);
    }

    #[test]
    fn test_write_word() {
        let mut mmu = MMU::new();
        mmu.write_word(0xC000, 0xFF11);
        assert_eq!(mmu.sram[0], 0x11);
        assert_eq!(mmu.sram[1], 0xFF);
    }

    #[test]
    fn test_push_stack() {
        let mut mmu = MMU::new();
        mmu.push_stack(0x11FF);
        mmu.push_stack(0x22DD);
        assert_eq!(mmu.sp, 0xDFFC); // 4 bytes are on the stack.

        // Written little endian, read_word reads as little endian and assembles back to a u16.
        assert_eq!(mmu.read_word(mmu.sp), 0x22DD);
        assert_eq!(mmu.read_word(mmu.sp + 2), 0x11FF);
    }

    #[test]
    fn test_pop_stack() {
        let mut mmu = MMU::new();
        mmu.push_stack(0x11FF);
        let value = mmu.pop_stack();
        assert_eq!(0x11FF, value);
        assert_eq!(mmu.sp, STACK_TOP + 1); // Stack Pointer has been reset.
    }

    /// Test setting the af register. Given each register is implemented using a macro, we only need
    /// to test one of them.
    #[test]
    fn test_af() {
        let mut mmu = MMU::new();
        mmu.a = 0xFF;
        mmu.f = 0x11;
        assert_eq!(mmu.af(), 0xFF11)
    }

    /// Test getting the af register. Given each register is implemented using a macro, we only need
    /// to test one of them.
    #[test]
    fn test_set_af() {
        let mut mmu = MMU::new();
        mmu.set_af(0xFF11);
        assert_eq!(mmu.a, 0xFF);
        assert_eq!(mmu.f, 0x11);
    }

    #[test]
    fn test_get_flags() {
        let mmu = &mut MMU::new();
        mmu.f = 0b10100000;
        assert_eq!(mmu.flag_z(), true);
        assert_eq!(mmu.flag_h(), true);
    }

    #[test]
    fn test_set_flags() {
        let mut mmu = MMU::new();
        mmu.set_flag_z(true);
        mmu.set_flag_n(true);
        mmu.set_flag_h(true);
        mmu.set_flag_c(true);
        assert_eq!(mmu.f, 0b11110000, "{:b}", mmu.f);

        mmu.set_flag_z(true);
        mmu.set_flag_n(true);
        mmu.set_flag_h(false);
        mmu.set_flag_c(false);
        assert_eq!(mmu.f, 0b11000000, "{:b}", mmu.f);
    }
}
