mod apu;
mod bootloader;
mod interrupts;
mod ppu;
mod registers;
mod timer;
use super::cartridge::Cartridge;
use apu::ApuRegisters;
use bootloader::{BootLoader, BOOTROM_MMU_VALUES};
use interrupts::Interrupts;
use ppu::PpuRegisters;
use timer::TimerRegisters;

pub struct MMU {
    hram: [u8; 0x7F], // 127 bytes of "High RAM" (DMA accessible) aka Zero page.
    oam: [u8; 0xA0],  // 160 bytes of OAM RAM.

    sram: [u8; 0x2000], // 8KB (no GBC banking support).
    vram: [u8; 0x2000], // 8KB graphics RAM.
    bootloader: BootLoader,
    pub ppu: PpuRegisters,
    apu: ApuRegisters,
    pub timer: TimerRegisters,

    cartridge: Cartridge, // Cartridge contains the MBC logic.
    pub gamepad: u8,
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
    pub fn new(cartridge_path: Option<&String>, use_bootrom: bool) -> Self {
        let mut mmu = Self {
            bootloader: BootLoader::new(use_bootrom),
            cartridge: Cartridge::new(cartridge_path),
            ppu: PpuRegisters::new(),
            apu: ApuRegisters::new(),
            interrupts: Interrupts::new(),
            timer: TimerRegisters::new(),
            hram: [0; 0x7F],
            oam: [0; 0xA0],
            sram: [0; 0x2000],
            vram: [0; 0x2000],
            gamepad: 0x2F, // Initialize with nothing pressed, bit 5 (buttons) selected.
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
        };

        // Initialize memory, timers, registers, etc. Typically the bootloader will do this, but if
        // we skip using the bootloader (probably becuase we don't have a ROM), we can just set the
        // end result.
        if !use_bootrom {
            println!("DONT USE BOOTROM");
            BOOTROM_MMU_VALUES
                .iter()
                .for_each(|(address, value)| mmu.wb(*address, *value));

            mmu.a = 0x01;
            mmu.f = 0xB0;
            mmu.b = 0x00;
            mmu.c = 0x13;
            mmu.d = 0x00;
            mmu.e = 0xD8;
            mmu.h = 0x01;
            mmu.l = 0x4D;
            mmu.pc = 0x0100;
            mmu.sp = 0xFFFE;
            // mmu.interrupts.intf = 1;
            // mmu.timer.divider = 147;
            // mmu.ppu.mode = 1;
            // mmu.ppu.line = 84;  (this seems to get closest, but it's still quite wrong).
            // mmu.ppu.obj_palette_0 = 0;
            // mmu.ppu.obj_palette_1 = 0;
        };

        mmu
    }

    /// Read a byte from address.
    pub fn rb(&self, address: u16) -> u8 {
        match address {
            // the first 256KB that's usually addressing the cartridge main memory bank initially
            // addresses the BootLoader.
            0x00..=0xFF => {
                if self.bootloader.is_enabled {
                    self.bootloader.rb(address)
                } else {
                    self.cartridge.rb(address)
                }
            }
            0x0000..=0x7FFF => self.cartridge.rb(address),
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],
            0xC000..=0xDFFF => self.sram[(address - 0xC000) as usize],
            0xE000..=0xFDFF => self.sram[(address - 0xC000 - 0x2000) as usize], // Mirror 0xC000.
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],
            0xFEA0..=0xFEFF => 0xFF,
            0xFF00 => self.gamepad,
            0xFF0f => self.interrupts.intf,
            0xFF01 => 0, // TODO: serial write.
            0xFF02 => 0, // TODO: serial control.
            0xFF04..=0xFF07 => self.timer.rb(address),
            0xFF10..=0xFF3F => self.apu.rb(address),
            0xFF46 => panic!("0xff46: OAM DMA cannot be read from."),
            0xFF40..=0xFF4B => self.ppu.rb(address),
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
            0x0000..=0x7FFF => self.cartridge.wb(address, value), // Cartridge control registers.
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = value,
            0xA000..=0xBFFF => self.cartridge.wb(address, value), // Possible cartridge RAM.
            0xC000..=0xDFFF => self.sram[(address - 0xC000) as usize] = value,
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = value,
            0xFEA0..=0xFEFF => (),
            0xFF00 => self.gamepad = value,
            0xFF01 => (),
            // 0xFF01 => println!("{}", value as char), // TODO: serial
            0xFF02 => (), // TODO: serial control.
            0xFF04..=0xFF07 => self.timer.wb(address, value),
            0xFF0F => self.interrupts.intf = value,
            0xFF10..=0xFF3F => self.apu.wb(address, value),
            0xFF46 => self.oam_dma(value),
            0xFF40..=0xFF4B => self.ppu.wb(address, value),
            0xFF50 => self.bootloader.is_enabled = false,
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

    /// If LY and LYC are equal and if LYC Interrupt enable (0xFF41) is set, set a STAT interrupt.
    /// Documentation says this is "permanently compared" so we should do it every tick. It's
    /// possible that it can be optimized. There's also a possibility it also has to be done
    /// immediately when ppu.lyc is changed.
    pub fn check_lyc_interrupt(&mut self) {
        if self.ppu.lyc_int_enable && self.ppu.line == self.ppu.lyc {
            self.interrupts.intf |= 0x02;
        }
    }
}

/// Return boolean state of a bit in a byte. This is for convenience and not a concept of the DMG-01
/// internals.
pub fn is_bit_set(value: u8, position: u8) -> bool {
    (value & (1 << position)) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_bit_set() {
        assert!(is_bit_set(0b10000000, 7));
        assert!(is_bit_set(0b11111111, 0));
        assert!(!is_bit_set(0b11111110, 0));
        assert!(!is_bit_set(0b10000000, 6));
    }

    #[test]
    fn test_rw() {
        let mut mmu = MMU::new(None, false);
        mmu.sram[0] = 0xFF;
        mmu.sram[1] = 0x11;
        let word = mmu.rw(0xC000);
        assert_eq!(word, 0x11FF);
    }

    #[test]
    fn test_ww() {
        let mut mmu = MMU::new(None, false);
        mmu.ww(0xC000, 0xFF11);
        assert_eq!(mmu.sram[0], 0x11);
        assert_eq!(mmu.sram[1], 0xFF);
    }

    #[test]
    fn test_push_stack() {
        let mut mmu = MMU::new(None, false);
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
        let mut mmu = MMU::new(None, false);
        mmu.sp = 0xfffe; // A common place to put the stack.
        mmu.push_stack(0x11FF);
        assert_eq!(mmu.sp, 0xfffc); // Stack Pointer has been decremented to the next address slot.
        let value = mmu.pop_stack();
        assert_eq!(0x11FF, value);
        assert_eq!(mmu.sp, 0xfffe); // Stack Pointer has been reset.
    }
}
