use super::opcode::OpCodes;

use super::alu::*;
use super::MMU;
use log::{error, info};
pub struct CPU {
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
            opcodes: OpCodes::from_path("data/opcodes.json").unwrap(),
        }
    }

    /// Perform a single opcode step and return how many cycles that took.
    /// Return the number of m-cycles required to perform the operation. This will be used for
    /// regulating how fast the CPU is emulated at.
    pub fn step(&mut self, mmu: &mut MMU) -> u8 {
        let op_address = mmu.pc; // Hold onto operation address before mutating it, for debugging.

        let mut opcode = mmu.get_next_byte();
        let is_cbprefix = opcode == 0xCB;

        // If the byte is not the opcode but actually the prefix, get another byte.
        if is_cbprefix {
            opcode = mmu.get_next_byte();
        }

        // The number of m-cycles required for this operation. This may be updated by an operation
        // if a conditional branch was NOT performed that costs less. We assume the condition is not
        // met.
        let mut cycles = self.opcodes.get_cycles(opcode, is_cbprefix, false);
        let mut condition_met = false;

        // Convenient register values at beginning of this opcode. This just reduces a lot of
        // repetitiveness in the opcodes below. Writing still requires mutably borrowing mmu.
        let MMU {
            a, b, c, e, h, pc, ..
        } = *mmu;

        let bc = mmu.bc();
        let de = mmu.de();
        let hl = mmu.hl();

        info!(
            "{} {:#x}",
            self.opcodes.get_opcode_repr(opcode, is_cbprefix),
            op_address
        );

        println!("{:x}", mmu.pc);

        // Match an opcode and manipulate memory accordingly.
        if !is_cbprefix {
            match opcode {
                0x04 => mmu.b = alu_inc(mmu, b),
                0x05 => mmu.b = alu_dec(mmu, b),
                0x06 => mmu.b = mmu.get_next_byte(),
                0x0C => mmu.c += 1,
                0x0D => mmu.c = alu_dec(mmu, c),
                0x0E => mmu.c = mmu.get_next_byte(),
                0x11 => {
                    let d16 = mmu.get_next_word();
                    mmu.set_de(d16);
                }
                0x13 => mmu.set_de(de.wrapping_add(1)),
                0x17 => {
                    // RLA is same as RL A but Z flag is unset.
                    mmu.a = alu_rl(mmu, a);
                    mmu.set_flag_z(false);
                }
                0x18 => {
                    let r8 = mmu.get_signed_byte(); // Must get first as it mutates PC.
                    mmu.pc = pc.wrapping_add(r8 as u16);
                }
                0x1A => mmu.a = mmu.rb(de),
                0x1E => mmu.e = mmu.get_next_byte(),
                0x20 => {
                    // Need to get byte to inc PC either way.
                    let r8 = mmu.get_signed_byte();
                    if !mmu.flag_z() {
                        mmu.pc = pc.wrapping_add(r8 as u16);
                        condition_met = true;
                    }
                }
                0x21 => {
                    let b = mmu.get_next_word();
                    mmu.set_hl(b)
                }
                0x22 => {
                    mmu.wb(hl, a);
                    let new_hl = hl.wrapping_add(1);
                    mmu.set_hl(new_hl);
                }
                0x23 => mmu.set_hl(hl.wrapping_add(1)),
                0x28 => {
                    let r8 = mmu.get_signed_byte() as u16;
                    if mmu.flag_z() {
                        mmu.pc = mmu.pc.wrapping_add(r8 as u16);
                        condition_met = true;
                    }
                }
                0x2E => mmu.l = mmu.get_next_byte(),
                0x31 => mmu.sp = mmu.get_next_word(),
                0x32 => {
                    mmu.wb(hl, a); // Set (HL) to A.
                    let new_hl = hl.wrapping_sub(1);
                    mmu.set_hl(new_hl); // Decrement.
                }
                0x3D => mmu.a = alu_dec(mmu, a),
                0x3E => mmu.a = mmu.get_next_byte(),
                0x4F => mmu.c = a,
                0x57 => mmu.d = a,
                0x67 => mmu.h = a,
                0x77 => mmu.wb(hl, a),
                0x7B => mmu.a = e,
                0x7C => mmu.a = h,
                0x9F => alu_sbc(mmu, a),
                0xAF => alu_xor(mmu, a),
                0xC1 => {
                    let address = mmu.pop_stack();
                    mmu.set_bc(address);
                }
                0xC5 => mmu.push_stack(bc),
                0xC9 => mmu.pc = mmu.pop_stack(),
                0xCD => {
                    let a16 = mmu.get_next_word(); // Advances mmu.pc to the next instruction.
                    mmu.push_stack(pc); // mmu.pc is the next instruction to be run.
                    mmu.pc = a16;
                }
                0xE0 => {
                    let addr = mmu.get_next_byte();
                    mmu.wb(0xFF00 + addr as u16, a);
                }
                0xE2 => mmu.wb(0xFF00 + c as u16, a),
                0xEA => {
                    let d8 = mmu.get_next_word();
                    mmu.wb(d8, a)
                }
                0xF0 => {
                    let addr = 0xFF00 + (mmu.get_next_byte() as u16);
                    mmu.a = mmu.rb(addr);
                }
                0xFE => {
                    let d8 = mmu.get_next_byte();
                    alu_cp(mmu, d8)
                }
                _ => self.panic_opcode(opcode, is_cbprefix, op_address),
            }
        } else {
            match opcode {
                0x7C => alu_bit(mmu, 7, h),
                0x11 => mmu.c = alu_rl(mmu, c),
                _ => self.panic_opcode(opcode, is_cbprefix, op_address),
            }
        }

        // Change cycles to be the smaller value (action not taken).
        if condition_met {
            cycles = self.opcodes.get_cycles(opcode, is_cbprefix, false);
        }

        cycles
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
