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
    pub fn step(&mut self, m: &mut MMU) -> u8 {
        let op_address = m.pc; // Hold onto operation address before mutating it, for debugging.

        let mut opcode = m.get_byte();
        let is_cbprefix = opcode == 0xCB;

        // If the byte is not the opcode but actually the prefix, get another byte.
        if is_cbprefix {
            opcode = m.get_byte();
        }

        // The number of m-cycles required for this operation. This may be updated by an operation
        // if a conditional branch was NOT performed that costs less. We assume the condition is not
        // met.
        let mut cycles = self.opcodes.get_cycles(opcode, is_cbprefix, false);
        let mut condition_met = false;

        // Convenient register values at beginning of this opcode.
        // TODO

        // Match an opcode and manipulate memory accordingly.
        if !is_cbprefix {
            match opcode {
                0x04 => m.b = alu_inc(m, m.b),
                0x05 => m.b = alu_dec(m, m.b),
                0x06 => m.b = m.get_byte(),
                0x0C => m.c += 1,
                0x0D => m.c = alu_dec(m, m.c),
                0x0E => m.c = m.get_byte(),
                0x11 => {
                    let d16 = m.get_word();
                    m.set_de(d16);
                }
                0x13 => m.set_de(m.de().wrapping_add(1)),
                0x17 => {
                    // RLA is same as RL A but Z flag is unset.
                    m.a = alu_rl(m, m.a);
                    m.set_flag_z(false);
                }
                0x18 => {
                    let r8 = m.get_signed_byte(); // Must get first as it mutates PC.
                    m.pc = m.pc.wrapping_add(r8 as u16);
                }
                0x1A => m.a = m.read_byte(m.de()),
                0x1E => m.e = m.get_byte(),
                0x20 => {
                    // Need to get byte to inc PC either way.
                    let r8 = m.get_signed_byte();
                    if !m.flag_z() {
                        m.pc = m.pc.wrapping_add(r8 as u16);
                        condition_met = true;
                    }
                }
                0x21 => {
                    let b = m.get_word();
                    m.set_hl(b)
                }
                0x22 => {
                    m.write(m.hl(), m.a);
                    let new_hl = m.hl().wrapping_add(1);
                    m.set_hl(new_hl);
                }
                0x23 => m.set_hl(m.hl().wrapping_add(1)),
                0x28 => {
                    let r8 = m.get_signed_byte() as u16;
                    if m.flag_z() {
                        m.pc = m.pc.wrapping_add(r8 as u16);
                        condition_met = true;
                    }
                }
                0x2E => m.l = m.get_byte(),
                0x31 => m.sp = m.get_word(),
                0x32 => {
                    m.write(m.hl(), m.a); // Set (HL) to A.
                    let new_hl = m.hl().wrapping_sub(1);
                    m.set_hl(new_hl); // Decrement.
                }
                0x3D => m.a = alu_dec(m, m.a),
                0x3E => m.a = m.get_byte(),
                0x4F => m.c = m.a,
                0x57 => m.d = m.a,
                0x67 => m.h = m.a,
                0x77 => m.write(m.hl(), m.a),
                0x7B => m.a = m.e,
                0x7C => m.a = m.h,
                0x9F => alu_sbc(m, m.a),
                0xAF => alu_xor(m, m.a),
                0xC1 => {
                    let address = m.pop_stack();
                    m.set_bc(address);
                }
                0xC5 => m.push_stack(m.bc()),
                0xC9 => m.pc = m.pop_stack(),
                0xCD => {
                    let a16 = m.get_word(); // Advances m.pc to the next instruction.
                    m.push_stack(m.pc); // m.pc is the next instruction to be run.
                    m.pc = a16;
                }
                0xE0 => {
                    let addr = m.get_byte();
                    m.write(0xFF00 + addr as u16, m.a);
                }
                0xE2 => m.write(0xFF00 + m.c as u16, m.a),
                0xEA => {
                    let d8 = m.get_word();
                    m.write(d8, m.a)
                }
                0xF0 => {
                    let addr = 0xFF00 + (m.get_byte() as u16);
                    m.a = m.read_byte(addr);
                }
                0xFE => {
                    let d8 = m.get_byte();
                    alu_cp(m, d8)
                }
                _ => self.panic_opcode(opcode, is_cbprefix, op_address),
            }
        } else {
            match opcode {
                0x7C => alu_bit(m, 7, m.h),
                0x11 => m.c = alu_rl(m, m.c),
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
