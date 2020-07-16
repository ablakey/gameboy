mod bootloader;
mod cartridge;
mod hwregisters;
mod registers;
use bootloader::BootLoader;
use cartridge::Cartridge;
use hwregisters::HardwareRegisters;

/// Memory map addresses
const HWREG_TOP: u16 = 0xFF7F;
const HWREG_BOT: u16 = 0xFF00;
const STACK_TOP: u16 = 0xDFFF; // Stack is put at the top of SRAM and grows downwards.
const SRAM_TOP: u16 = 0xDFFF;
const SRAM_BOT: u16 = 0xC000;
// const CART_RAM_TOP: u16 = 0xBFFF; // Aka Switchable RAM bank (RAM that lives. in a cartridge)
// const CART_RAM_BOT: u16 = 0xA000;
const CART_ROM_TOP: u16 = 0x7FFF; // Range includes parts of cartridge like interrupt vectors.
const CART_ROM_BOT: u16 = 0x0000;
const VRAM_TOP: u16 = 0x9FFF;
const VRAM_BOT: u16 = 0x8000;

// TODO explain (that MMU has memory, registers, io regsiters (TBD) and other state)
pub struct MMU {
    sram: [u8; 0x2000], // 8KB of DMG-01 memory.
    vram: [u8; 0x2000], // 8KB of DMG-01 memory.
    boot: BootLoader,
    hwreg: HardwareRegisters,
    cart: Cartridge,
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
    /// Initialize the MMU by loading the boot_rom into the first 256 addressable bytes.
    pub fn new() -> Self {
        Self {
            sram: [0; 0x2000],
            vram: [0; 0x2000],
            boot: BootLoader::new(),
            hwreg: HardwareRegisters::new(),
            cart: Cartridge::new(),
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
    pub fn rb(&self, address: u16) -> u8 {
        match address {
            0x00..=0xFF => self.boot.rb(address), // When bootloader is done, need to remap.
            SRAM_BOT..=SRAM_TOP => self.sram[(address - SRAM_BOT) as usize],
            VRAM_BOT..=VRAM_TOP => self.vram[(address - VRAM_BOT) as usize],
            CART_ROM_BOT..=CART_ROM_TOP => self.cart.rb(address),
            _ => panic!("Tried to read from {:#x} which is not mapped.", address),
        }
    }

    /// Write an 8-bit value to an address.
    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            HWREG_BOT..=HWREG_TOP => self.hwreg.set(address, value),
            VRAM_BOT..=VRAM_TOP => self.vram[(address - VRAM_BOT) as usize] = value,
            SRAM_BOT..=SRAM_TOP => self.sram[(address - SRAM_BOT) as usize] = value,
            _ => panic!("Tried to write to {:#x} which is not mapped.", address),
        }
    }

    /// Read a word from address.
    /// DMG-01 is little endian so the least-significant byte is read first.
    pub fn rw(&self, address: u16) -> u16 {
        let lsb = self.rb(address) as u16;
        let msb = self.rb(address + 1) as u16;
        (msb << 8) | lsb
    }

    /// Write a 16-bit value to an address and the immediate address after.
    /// DMG-01 is little endian so the least-significant byte is written first.
    pub fn ww(&mut self, address: u16, value: u16) {
        self.wb(address, (value & 0xFF) as u8); // Mask only the LSB.
        self.wb(address + 1, (value >> 8) as u8); // bit-shift until we have only the MSB.
    }

    /// Get the next byte and advance the program counter by 1.
    pub fn get_next_byte(&mut self) -> u8 {
        let byte = self.rb(self.pc);
        self.pc += 1;
        byte
    }

    /// Get the next byte as a two's complement signed integer and advance the program counter by 1.
    pub fn get_signed_byte(&mut self) -> i8 {
        self.get_next_byte() as i8
    }

    /// Get the next word in memory and advance the program counter by 2.
    pub fn get_next_word(&mut self) -> u16 {
        let word = self.rw(self.pc);
        self.pc += 2;
        word
    }

    /// Push a word (an address of the an instruction) to the stack.
    /// Stack decrements by one first (it grows downward in address space at the top of low RAM).
    pub fn push_stack(&mut self, address: u16) {
        self.sp -= 2;
        self.ww(self.sp, address);
    }

    /// Pop a word off the stack.
    /// It will go into a register.
    pub fn pop_stack(&mut self) -> u16 {
        let address = self.rw(self.sp);
        self.sp += 2;
        address
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rw() {
        let mut mmu = MMU::new();
        mmu.sram[0] = 0xFF;
        mmu.sram[1] = 0x11;
        let word = mmu.rw(0xC000);
        assert_eq!(word, 0x11FF);
    }

    #[test]
    fn test_ww() {
        let mut mmu = MMU::new();
        mmu.ww(0xC000, 0xFF11);
        assert_eq!(mmu.sram[0], 0x11);
        assert_eq!(mmu.sram[1], 0xFF);
    }

    #[test]
    fn test_push_stack() {
        let mut mmu = MMU::new();
        mmu.push_stack(0x11FF);
        mmu.push_stack(0x22DD);
        assert_eq!(mmu.sp, 0xDFFC); // 4 bytes are on the stack.

        // Written little endian, rw reads as little endian and assembles back to a u16.
        assert_eq!(mmu.rw(mmu.sp), 0x22DD);
        assert_eq!(mmu.rw(mmu.sp + 2), 0x11FF);
    }

    #[test]
    fn test_pop_stack() {
        let mut mmu = MMU::new();
        mmu.push_stack(0x11FF);
        let value = mmu.pop_stack();
        assert_eq!(0x11FF, value);
        assert_eq!(mmu.sp, STACK_TOP + 1); // Stack Pointer has been reset.
    }
}
