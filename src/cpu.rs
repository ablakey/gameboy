use super::opcode::OpCodes;
use super::registers::Registers;
use super::MMU;
use log::{error, info};
pub struct CPU {
    pc: u16,
    sp: u16,
    mmu: MMU,
    reg: Registers,
    opcodes: OpCodes,
}

impl CPU {
    /// Initialise the CPU, its pointers, MMU, and registers.
    ///
    /// The Stack:
    /// The stack pointer begins one word above the topmost address allocated to the stack. It
    /// decrements automatically when used, so first use will push to stack at 0xDFFF. Stack
    /// increases downwards. http://gameboy.mongenel.com/dmg/asmmemmap.html explains that high ram
    /// was originally meant for the stack. By starting at 0xE001 we can decrement by 2 addresses
    /// given every address put on the stack is a word in length.
    ///
    /// Program Counter:
    /// Begins at 0x0 and runs through the bootloader. Once the bootloader is complete, it should
    /// be at 0x100. Some emulators ignore the bootloader, pre-initialize the emulator, and begin
    /// at 0x100. We don't take that shortcut, as running the bootloader is a great test.
    pub fn new() -> Self {
        Self {
            pc: 0,
            sp: 0xE001, // Stack increases downwards. Start one above the allocated stack space.
            mmu: MMU::new(),
            reg: Registers::new(),
            opcodes: OpCodes::from_path("data/opcodes.json").unwrap(),
        }
    }

    /// Perform a single opcode step and return how many cycles that took.
    /// Return the number of m-cycles required to perform the operation. This will be used for
    /// regulating how fast the CPU is emulated at.
    pub fn step(&mut self) -> u8 {
        let op_address = self.pc; // Hold onto operation address before mutating it, for debugging.

        let mut opcode = self.get_byte();
        let is_cbprefix = opcode == 0xCB;

        // If the byte is not the opcode but actually the prefix, get another byte.
        if is_cbprefix {
            opcode = self.get_byte();
        }

        // The number of m-cycles required for this operation. This may be updated by an operation
        // if a conditional branch was NOT performed that costs less. We assume the condition is not
        // met.
        let mut cycles = self.opcodes.get_cycles(opcode, is_cbprefix, false);
        let mut condition_met = false;

        // Match an opcode and manipulate memory accordingly.
        if !is_cbprefix {
            match opcode {
                0x04 => self.reg.b = self.reg.alu_inc(self.reg.b),
                0x05 => self.reg.b = self.reg.alu_dec(self.reg.b),
                0x06 => self.reg.b = self.get_byte(),
                0x0C => self.reg.c += 1,
                0x0D => self.reg.c = self.reg.alu_dec(self.reg.c),
                0x0E => self.reg.c = self.get_byte(),
                0x11 => {
                    let d16 = self.get_word();
                    self.reg.set_de(d16);
                }
                0x13 => self.reg.set_de(self.reg.de().wrapping_add(1)),
                0x17 => {
                    // RLA is same as RL A but Z flag is unset.
                    self.reg.a = self.reg.alu_rl(self.reg.a);
                    self.reg.set_flag_z(false);
                }
                0x18 => {
                    let r8 = self.get_signed_byte(); // Must get first as it mutates PC.
                    self.pc = self.pc.wrapping_add(r8 as u16);
                }
                0x1A => self.reg.a = self.mmu.read_byte(self.reg.de()),
                0x1E => self.reg.e = self.get_byte(),
                0x20 => {
                    // Need to get byte to inc PC either way.
                    let r8 = self.get_signed_byte();
                    if !self.reg.flag_z() {
                        self.pc = self.pc.wrapping_add(r8 as u16);
                        condition_met = true;
                    }
                }
                0x21 => {
                    let b = self.get_word();
                    self.reg.set_hl(b)
                }
                0x22 => {
                    self.mmu.write(self.reg.hl(), self.reg.a);
                    let new_hl = self.reg.hl().wrapping_add(1);
                    self.reg.set_hl(new_hl);
                }
                0x23 => self.reg.set_hl(self.reg.hl().wrapping_add(1)),
                0x28 => {
                    let r8 = self.get_signed_byte() as u16;
                    if self.reg.flag_z() {
                        self.pc = self.pc.wrapping_add(r8 as u16);
                        condition_met = true;
                    }
                }
                0x2E => self.reg.l = self.get_byte(),
                0x31 => self.sp = self.get_word(),
                0x32 => {
                    self.mmu.write(self.reg.hl(), self.reg.a); // Set (HL) to A.
                    let new_hl = self.reg.hl().wrapping_sub(1);
                    self.reg.set_hl(new_hl); // Decrement.
                }
                0x3D => self.reg.a = self.reg.alu_dec(self.reg.a),
                0x3E => self.reg.a = self.get_byte(),
                0x4F => self.reg.c = self.reg.a,
                0x57 => self.reg.d = self.reg.a,
                0x67 => self.reg.h = self.reg.a,
                0x77 => self.mmu.write(self.reg.hl(), self.reg.a),
                0x7B => self.reg.a = self.reg.e,
                0x7C => self.reg.a = self.reg.h,
                0x9F => self.reg.alu_sbc(self.reg.a),
                0xAF => self.reg.alu_xor(self.reg.a),
                0xC1 => {
                    let address = self.pop_stack();
                    self.reg.set_bc(address);
                }
                0xC5 => self.push_stack(self.reg.bc()),
                0xC9 => self.pc = self.pop_stack(),
                0xCD => {
                    let a16 = self.get_word(); // Advances self.pc to the next instruction.
                    self.push_stack(self.pc); // self.pc is the next instruction to be run.
                    self.pc = a16;
                }
                0xE0 => {
                    let addr = self.get_byte();
                    self.mmu.write(0xFF00 + addr as u16, self.reg.a);
                }
                0xE2 => self.mmu.write(0xFF00 + self.reg.c as u16, self.reg.a),
                0xEA => {
                    let d8 = self.get_word();
                    self.mmu.write(d8, self.reg.a)
                }
                0xFE => {
                    let d8 = self.get_byte();
                    self.reg.alu_cp(d8)
                }
                _ => self.panic_opcode(opcode, is_cbprefix, op_address),
            }
        } else {
            match opcode {
                0x7C => self.reg.alu_bit(7, self.reg.h),
                0x11 => self.reg.c = self.reg.alu_rl(self.reg.c),
                _ => self.panic_opcode(opcode, is_cbprefix, op_address),
            }
        }

        // Change cycles to be the smaller value (action not taken).
        if condition_met {
            cycles = self.opcodes.get_cycles(opcode, is_cbprefix, false);
        }

        info!(
            "{} {:#x}",
            self.opcodes.get_opcode_repr(opcode, is_cbprefix),
            op_address
        );

        cycles
    }

    /// Push a word (an address of the an instruction) to the stack.
    /// Stack decrements by one first (it grows downward in address space at the top of low RAM).
    fn push_stack(&mut self, address: u16) {
        self.sp -= 2;
        self.mmu.write_word(self.sp, address);
    }

    /// Pop a word off the stack.
    /// It will go into a register.
    fn pop_stack(&mut self) -> u16 {
        let address = self.mmu.read_word(self.sp);
        self.sp += 2;
        address
    }

    /// Get the next byte and advance the program counter by 1.
    fn get_byte(&mut self) -> u8 {
        let byte = self.mmu.read_byte(self.pc);
        self.pc += 1;
        byte
    }

    /// Get the next byte as a two's complement signed integer and advance the program counter by 1.
    fn get_signed_byte(&mut self) -> i8 {
        self.get_byte() as i8
    }

    /// Get the next word in memory and advance the program counter by 2.
    fn get_word(&mut self) -> u16 {
        let word = self.mmu.read_word(self.pc);
        self.pc += 2;
        word
    }

    /// Debug function. Panic when an opcode is not handled.
    fn panic_opcode(&self, opcode: u8, is_cbprefix: bool, operation_address: u16) {
        let msg = format!(
            "{} {:#06x}",
            self.opcodes.get_opcode_repr(opcode, is_cbprefix),
            operation_address
        );
        error!("{}", msg);
        panic!("{}", msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_stack() {
        let mut cpu = CPU::new();
        cpu.push_stack(0x11FF);
        cpu.push_stack(0x22DD);
        assert_eq!(cpu.sp, 0xDFFD); // 4 bytes are on the stack: 0xE001 - 0x0004;

        // Written little endian, read_word reads as little endian and assembles back to a u16.
        assert_eq!(cpu.mmu.read_word(cpu.sp), 0x22DD);
        assert_eq!(cpu.mmu.read_word(cpu.sp + 2), 0x11FF);
    }

    #[test]
    fn test_pop_stack() {
        let mut cpu = CPU::new();
        cpu.push_stack(0x11FF);
        let value = cpu.pop_stack();
        assert_eq!(0x11FF, value);
        assert_eq!(cpu.sp, 0xE001); // Stack Pointer has been reset.
    }
}
