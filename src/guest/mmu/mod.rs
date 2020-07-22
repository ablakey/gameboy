mod bootloader;
mod cartridge;
mod hwreg;
mod reg;
use bootloader::BootLoader;
use cartridge::Cartridge;
use hwreg::HardwareRegisters;
use log::info;

/// Memory map addresses
const HRAM_TOP: u16 = 0xFFFE;
const HRAM_BOT: u16 = 0xFF80;
const HWREG_TOP: u16 = 0xFF7F;
const HWREG_BOT: u16 = 0xFF00;
const SRAM_TOP: u16 = 0xDFFF;
const SRAM_BOT: u16 = 0xC000;
const VRAM_TOP: u16 = 0x9FFF;
const VRAM_BOT: u16 = 0x8000;
const CART_ROM_TOP: u16 = 0x7FFF; // Range includes parts of cartridge like interrupt vectors.
const CART_ROM_BOT: u16 = 0x0000;

// TODO explain (that MMU has memory, registers, io regsiters (TBD) and other state)
pub struct MMU {
    hram: [u8; 0x7F],   // 127 bytes of "High RAM" (DMA accessible) aka Zero page.
    sram: [u8; 0x2000], // 8KB (no GBC banking support).
    vram: [u8; 0x2000], // 8KB graphics RAM.
    boot: BootLoader,
    pub hwreg: HardwareRegisters,
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
            hram: [0; 0x7F],
            sram: [0; 0x2000],
            vram: [0; 0x2000],
            boot: BootLoader::new(),
            hwreg: HardwareRegisters::new(),
            cart: Cartridge::new(),
            pc: 0,
            sp: 0, // Initialized by the software.
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
            0xFF46 => panic!("0xff46: OAM DMA cannot be read from."),
            0x00..=0xFF => self.boot.rb(address), // When bootloader is done, need to remap.
            HWREG_BOT..=HWREG_TOP => self.hwreg.get(address), // Some are not readable.
            HRAM_BOT..=HRAM_TOP => self.hram[(address - HRAM_BOT) as usize],
            SRAM_BOT..=SRAM_TOP => self.sram[(address - SRAM_BOT) as usize],
            VRAM_BOT..=VRAM_TOP => self.vram[(address - VRAM_BOT) as usize],
            CART_ROM_BOT..=CART_ROM_TOP => self.cart.rb(address),
            _ => panic!("Tried to read from {:#x} which is not mapped.", address),
        }
    }

    /// Write an 8-bit value to an address.
    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            0xFF46 => self.oam_dma(address),
            HWREG_BOT..=HWREG_TOP => self.hwreg.set(address, value), // Some are not writable.
            HRAM_BOT..=HRAM_TOP => self.hram[(address - HRAM_BOT) as usize] = value,
            SRAM_BOT..=SRAM_TOP => self.sram[(address - SRAM_BOT) as usize] = value,
            VRAM_BOT..=VRAM_TOP => self.vram[(address - VRAM_BOT) as usize] = value,
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
        info!("{:#04x}", byte);
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
        info!("{:#06x}", word);
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

    pub fn oam_dma(&mut self, _address: u16) {
        // TODO: write 160 bytes from address -> OAM RAM.
        // Assert that address is a multiple of 0x100  address % 0x100 == 0
        // Write tests that set up some memory to be copied, performs a copy, and checks that it was
        // copied. Can probably just set a byte at address and a byte at address + 159
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
        mmu.sp = 0xDFFF;
        mmu.push_stack(0x11FF);
        mmu.push_stack(0x22DD);
        assert_eq!(mmu.sp, 0xDFFB); // 4 bytes are on the stack.

        // Written little endian, rw reads as little endian and assembles back to a u16.
        assert_eq!(mmu.rw(mmu.sp), 0x22DD);
        assert_eq!(mmu.rw(mmu.sp + 2), 0x11FF);
    }

    #[test]
    fn test_pop_stack() {
        let mut mmu = MMU::new();
        mmu.sp = 0xfffe; // A common place to put the stack.
        mmu.push_stack(0x11FF);
        assert_eq!(mmu.sp, 0xfffc); // Stack Pointer has been decremented to the next address slot.
        let value = mmu.pop_stack();
        assert_eq!(0x11FF, value);
        assert_eq!(mmu.sp, 0xfffe); // Stack Pointer has been reset.
    }
}
