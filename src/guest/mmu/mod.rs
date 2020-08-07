mod apu;
mod bootrom;
mod cartridge;
mod cpu;
mod interrupts;
mod ppu;
use crate::debug;
use apu::ApuRegisters;
use bootrom::BootRom;
use cartridge::Cartridge;
use interrupts::Interrupts;
use ppu::PpuRegisters;
use std::panic;

pub struct MMU {
    hram: [u8; 0x7F],   // 127 bytes of "High RAM" (DMA accessible) aka Zero page.
    oam: [u8; 0xA0],    // 160 bytes of OAM RAM.
    cram: [u8; 0x2000], // 8KB switchable cartridge RAM.
    sram: [u8; 0x2000], // 8KB (no GBC banking support).
    vram: [u8; 0x2000], // 8KB graphics RAM.
    bootrom: BootRom,
    pub ppureg: PpuRegisters,
    apureg: ApuRegisters,
    cartridge: Cartridge,
    pub interrupts: Interrupts,
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

impl MMU {
    /// Initialize the MMU by loading the boot_rom into the first 256 addressable bytes.
    pub fn new(cartridge_path: Option<&String>) -> Self {
        Self {
            bootrom: BootRom::new(),
            cartridge: Cartridge::new(cartridge_path),
            ppureg: PpuRegisters::new(),
            apureg: ApuRegisters::new(),
            interrupts: Interrupts::new(),
            hram: [0; 0x7F],
            oam: [0; 0xA0],
            cram: [0; 0x2000],
            sram: [0; 0x2000],
            vram: [0; 0x2000],
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
            0x00..=0xFF => {
                if self.bootrom.is_enabled {
                    self.bootrom.rb(address)
                } else {
                    self.cartridge.rb(address)
                }
            }
            0x0000..=0x7FFF => self.cartridge.rb(address),
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],
            0xC000..=0xDFFF => self.sram[(address - 0xC000) as usize],
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],
            0xFEA0..=0xFEFF => 0xFF,
            0xFF00 => 0xFF, // TODO: gamepad.
            0xFF01 => 0,    // TODO: serial write.
            0xFF02 => 0,    // TODO: serial control.
            0xFF04 => 0,    // TODO: Divider timer register.
            0xFF05 => 0,    // TODO: Timer Counter.
            0xFF06 => 0,    // TODO: Timer Modulo.
            0xFF07 => 0,    // TODO: Timer control.
            0xFF0F => 0,    // TODO: Interrupt Flag (IF)
            0xFF10..=0xFF3F => self.apureg.rb(address),
            0xFF46 => panic!("0xff46: OAM DMA cannot be read from."),
            0xFF40..=0xFF4B => self.ppureg.rb(address),
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.interrupts.inte,
            _ => {
                panic!("Tried to read from {:#x} which is not mapped.", address);
            }
        }
    }

    /// Write an 8-bit value to an address.
    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => self.cartridge.wb(address, value),
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = value,
            0xA000..=0xBFFF => self.cram[(address - 0xA000) as usize] = value,
            0xC000..=0xDFFF => self.sram[(address - 0xC000) as usize] = value,
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = value,
            0xFEA0..=0xFEFF => (),
            0xFF00 => (), // TODO: gamepad.
            0xFF01 => (), // TODO: serial write.
            0xFF02 => (), // TODO: serial control.
            0xFF04 => (), // TODO: Divider timer register.
            0xFF05 => (), // TODO: Timer Counter.
            0xFF06 => (), // TODO: Timer Modulo.
            0xFF07 => (), // TODO: Timer control.
            0xFF0F => self.interrupts.intf = value,
            0xFF10..=0xFF3F => self.apureg.wb(address, value),
            0xFF46 => self.oam_dma(value),
            0xFF40..=0xFF4B => self.ppureg.wb(address, value),
            0xFF50 => self.bootrom.is_enabled = false,
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            0xFF7F => (), // tetris.gb off-by-one error.
            0xFFFF => self.interrupts.inte = value,
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

    /// A very simple write of 160 bytes beginning at an address into OAM memory.
    /// The value is actually the MSB of the address. From there we walk 160 bytes from it and
    /// copy them to OAM.
    pub fn oam_dma(&mut self, value: u8) {
        let base = (value as u16) << 8;
        for n in 0..0xA0 {
            let byte = self.rb(base + n);
            self.wb(0xFE00 + n, byte);
        }
    }

    /// Panic with a given message, but also printout some debug info.
    /// By making it a diverging function, we don't care about return type.
    pub fn dump_state(&self) {
        // Dump VRAM
        let vram_dump = debug::format_hex(&self.vram.to_vec(), 0x8000);
        debug::dump_to_file(vram_dump, "vram");

        // Dump SRAM
        let vram_dump = debug::format_hex(&self.sram.to_vec(), 0xC000);
        debug::dump_to_file(vram_dump, "sram");

        // Dump tilemaps
        let tilemap0 = (0x9800 - 0x8000) as usize;
        let tilemap0_dump = debug::format_tilemap(&self.vram[tilemap0..tilemap0 + 1024]);
        debug::dump_to_file(tilemap0_dump, "tilemap0");

        let tilemap1 = (0x9C00 - 0x8000) as usize;
        let tilemap0_dump = debug::format_tilemap(&self.vram[tilemap1..tilemap1 + 1024]);
        debug::dump_to_file(tilemap0_dump, "tilemap1");
    }

    /// Try to handle an interrupt and return the number of cycles it took.
    /// Usually this is 0 cycles and no interrupt is handled.
    pub fn try_interrupt(&mut self) -> u8 {
        match self.interrupts.try_interrupt() {
            None => 0,
            Some(n) if n < 5 => {
                // Addresses are 0x0040, 0x0048, 0x0050, 0x0058, 0x0060. By shifting by 3,
                // We can append that multiple of 8 to 0x0040.
                let address = 0x0040 + (n << 3) as u16;

                self.push_stack(self.pc);
                self.pc = address;

                4 // All interupts take 4 cycles to jump to. The actual routine will be longer.
            }
            Some(n) => panic!("Handled invalid interrupt flag: {:#b}", n),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rw() {
        let mut mmu = MMU::new(None);
        mmu.sram[0] = 0xFF;
        mmu.sram[1] = 0x11;
        let word = mmu.rw(0xC000);
        assert_eq!(word, 0x11FF);
    }

    #[test]
    fn test_ww() {
        let mut mmu = MMU::new(None);
        mmu.ww(0xC000, 0xFF11);
        assert_eq!(mmu.sram[0], 0x11);
        assert_eq!(mmu.sram[1], 0xFF);
    }

    #[test]
    fn test_push_stack() {
        let mut mmu = MMU::new(None);
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
        let mut mmu = MMU::new(None);
        mmu.sp = 0xfffe; // A common place to put the stack.
        mmu.push_stack(0x11FF);
        assert_eq!(mmu.sp, 0xfffc); // Stack Pointer has been decremented to the next address slot.
        let value = mmu.pop_stack();
        assert_eq!(0x11FF, value);
        assert_eq!(mmu.sp, 0xfffe); // Stack Pointer has been reset.
    }
}
