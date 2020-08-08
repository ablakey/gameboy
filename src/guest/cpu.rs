use super::opcode::OpCodes;

use super::alu::*;
use super::MMU;
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
    /// Begins at 0x0 and runs through the bootrom. Once the bootrom is complete, it should
    /// be at 0x100. Some emulators ignore the bootrom, pre-initialize the emulator, and begin
    /// at 0x100. We don't take that shortcut, as running the bootrom is a great test.
    pub fn new() -> Self {
        Self {
            opcodes: OpCodes::from_path("data/opcodes.json").unwrap(),
        }
    }

    /// Perform a single opcode step and return how many cycles that took.
    /// Return the number of m-cycles required to perform the operation. This will be used for
    /// regulating how fast the CPU is emulated at.
    pub fn do_opcode(&self, mmu: &mut MMU) -> u8 {
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
        // Note: these are only valid because they aren't mutated more than once in an operation.
        // The PC, for example, might be incremented in an operation, and then read from. Therefore
        // getting the value of PC now would be a problem.
        let MMU {
            a,
            b,
            c,
            d,
            e,
            h,
            l,
            ..
        } = *mmu;

        let flag_z = mmu.flag_z();

        let af = mmu.af();
        let bc = mmu.bc();
        let de = mmu.de();
        let hl = mmu.hl();

        // Match an opcode and manipulate memory accordingly.
        if !is_cbprefix {
            match opcode {
                0x00 => (), // NOP
                0x01 => {
                    let d16 = mmu.get_next_word();
                    mmu.set_bc(d16);
                }
                0x03 => mmu.set_bc(bc.wrapping_add(1)),
                0x04 => mmu.b = alu_inc(mmu, b),
                0x05 => mmu.b = alu_dec(mmu, b),
                0x06 => mmu.b = mmu.get_next_byte(),
                0x09 => alu_add_16(mmu, bc),
                0x0A => mmu.a = mmu.rb(bc),
                0x0B => mmu.set_bc(bc.wrapping_sub(1)),
                0x0C => mmu.c += 1,
                0x0D => mmu.c = alu_dec(mmu, c),
                0x0E => mmu.c = mmu.get_next_byte(),
                0x11 => {
                    let d16 = mmu.get_next_word();
                    mmu.set_de(d16);
                }
                0x12 => mmu.wb(de, a),
                0x13 => mmu.set_de(de.wrapping_add(1)),
                0x15 => mmu.d = alu_dec(mmu, d),
                0x16 => mmu.d = mmu.get_next_byte(),
                0x17 => {
                    // RLA is same as RL A but Z flag is unset.
                    mmu.a = alu_rl(mmu, a);
                    mmu.set_flag_z(false);
                }
                0x18 => {
                    let r8 = mmu.get_signed_byte(); // Must get first as it mutates PC.
                    mmu.pc = mmu.pc.wrapping_add(r8 as u16);
                }
                0x19 => alu_add_16(mmu, de),
                0x1A => mmu.a = mmu.rb(de),
                0x1C => mmu.e = alu_inc(mmu, e),
                0x1D => mmu.e = alu_dec(mmu, e),
                0x1E => mmu.e = mmu.get_next_byte(),
                0x20 => {
                    let r8 = mmu.get_signed_byte(); // Need to get byte to inc PC either way.
                    if !mmu.flag_z() {
                        mmu.pc = mmu.pc.wrapping_add(r8 as u16);
                        condition_met = true;
                    }
                }
                0x21 => {
                    let b = mmu.get_next_word();
                    mmu.set_hl(b)
                }
                0x22 => {
                    mmu.wb(hl, a);
                    mmu.set_hl(hl.wrapping_add(1));
                }
                0x23 => mmu.set_hl(hl.wrapping_add(1)),
                0x24 => mmu.h = alu_inc(mmu, h),
                0x26 => mmu.h = mmu.get_next_byte(),
                0x28 => {
                    let r8 = mmu.get_signed_byte() as u16;
                    if mmu.flag_z() {
                        mmu.pc = mmu.pc.wrapping_add(r8 as u16);
                        condition_met = true;
                    }
                }
                0x2A => {
                    mmu.a = mmu.rb(hl);
                    mmu.set_hl(hl.wrapping_add(1));
                }
                0x2C => mmu.l = alu_inc(mmu, l),
                0x2D => mmu.l = alu_dec(mmu, l),
                0x2E => mmu.l = mmu.get_next_byte(),
                0x2F => alu_cpl(mmu),
                0x31 => {
                    let w = mmu.get_next_word();
                    mmu.sp = w
                }
                0x32 => {
                    mmu.wb(hl, a); // Set (HL) to A.
                    let new_hl = hl.wrapping_sub(1);
                    mmu.set_hl(new_hl); // Decrement.
                }
                0x34 => {
                    let value = alu_inc(mmu, mmu.rb(hl));
                    mmu.wb(hl, value);
                }
                0x35 => {
                    let value = alu_dec(mmu, mmu.rb(hl));
                    mmu.wb(hl, value);
                }
                0x36 => {
                    let d8 = mmu.get_next_byte();
                    mmu.wb(hl, d8);
                }
                0x3A => {
                    mmu.a = mmu.rb(hl);
                    mmu.set_hl(hl.wrapping_sub(1));
                }
                0x3C => mmu.a = alu_inc(mmu, a),
                0x3D => mmu.a = alu_dec(mmu, a),
                0x3E => mmu.a = mmu.get_next_byte(),
                0x40 => (), // LD B, B == NOP.
                0x4E => mmu.c = mmu.rb(hl),
                0x46 => mmu.b = mmu.rb(hl),
                0x47 => mmu.b = a,
                0x49 => (), // LD C, C == NOP.
                0x4F => mmu.c = a,
                0x50 => mmu.d = b,
                0x51 => mmu.d = c,
                0x52 => (), // LD D, D == NOP.
                0x53 => mmu.d = e,
                0x54 => mmu.d = h,
                0x55 => mmu.d = l,
                0x56 => mmu.d = mmu.rb(hl),
                0x57 => mmu.d = a,
                0x58 => mmu.a = b,
                0x59 => mmu.a = c,
                0x5A => mmu.a = d,
                0x5B => mmu.a = e,
                0x5C => mmu.a = h,
                0x5D => mmu.e = l,
                0x5E => mmu.e = mmu.rb(hl),
                0x5F => mmu.e = a,
                0x60 => mmu.h = b,
                0x61 => mmu.h = c,
                0x62 => mmu.h = d,
                0x63 => mmu.h = e,
                0x64 => mmu.h = h,
                0x65 => mmu.h = l,
                0x67 => mmu.h = a,
                0x68 => mmu.l = b,
                0x69 => mmu.l = c,
                0x6A => mmu.l = d,
                0x6B => mmu.l = e,
                0x6C => mmu.l = h,
                0x6D => mmu.l = l,
                0x6F => mmu.l = a,
                0x70 => mmu.wb(hl, b),
                0x71 => mmu.wb(hl, c),
                0x72 => mmu.wb(hl, d),
                0x73 => mmu.wb(hl, e),
                0x74 => mmu.wb(hl, h),
                0x75 => mmu.wb(hl, l),
                0x77 => mmu.wb(hl, a),
                0x78 => mmu.a = b,
                0x79 => mmu.a = c,
                0x7A => mmu.a = d,
                0x7B => mmu.a = e,
                0x7C => mmu.a = h,
                0x7D => mmu.a = l,
                0x7E => mmu.a = mmu.rb(hl),
                0x80 => alu_add(mmu, b),
                0x81 => alu_add(mmu, c),
                0x82 => alu_add(mmu, d),
                0x83 => alu_add(mmu, e),
                0x84 => alu_add(mmu, h),
                0x85 => alu_add(mmu, l),
                0x86 => alu_add(mmu, mmu.rb(hl)),
                0x87 => alu_add(mmu, a),
                0x90 => alu_sub(mmu, b),
                0x9F => alu_sbc(mmu, a),
                0xA1 => alu_and(mmu, c),
                0xA7 => alu_and(mmu, a),
                0xA9 => alu_xor(mmu, c),
                0xAF => alu_xor(mmu, a),
                0xB0 => alu_or(mmu, b),
                0xB1 => alu_or(mmu, c),
                0xBE => {
                    let value = mmu.rb(hl);
                    alu_cp(mmu, value);
                }
                0xC0 => {
                    if !flag_z {
                        mmu.pc = mmu.pop_stack();
                        condition_met = true;
                    }
                }
                0xC1 => {
                    let address = mmu.pop_stack();
                    mmu.set_bc(address);
                }
                0xC2 => {
                    let address = mmu.get_next_word(); // Need to get regardless to advance PC.
                    if !flag_z {
                        mmu.pc = address;
                        condition_met = true;
                    }
                }
                0xC3 => mmu.pc = mmu.get_next_word(),
                0xC5 => mmu.push_stack(bc),
                0xC6 => {
                    let value = mmu.get_next_byte();
                    alu_add(mmu, value);
                }
                0xC8 => {
                    if flag_z {
                        mmu.pc = mmu.pop_stack();
                        condition_met = true;
                    }
                }
                0xC9 => mmu.pc = mmu.pop_stack(),
                0xCA => {
                    let address = mmu.get_next_word(); // Need to get regardless to advance PC.
                    if flag_z {
                        mmu.pc = address;
                        condition_met = true;
                    }
                }
                0xCD => {
                    let a16 = mmu.get_next_word(); // Advances mmu.pc to the next instruction.
                    mmu.push_stack(mmu.pc); // mmu.pc is the next instruction to be run.
                    mmu.pc = a16;
                }
                0xD1 => {
                    let value = mmu.pop_stack();
                    mmu.set_de(value);
                }
                0xD5 => mmu.push_stack(de),
                0xD9 => {
                    mmu.pc = mmu.pop_stack();
                    mmu.interrupts.enable_ime(1); // RETI re-enables IME after this opcode.
                }
                0xE0 => {
                    let addr = mmu.get_next_byte();
                    mmu.wb(0xFF00 + addr as u16, a);
                }
                0xE1 => {
                    let value = mmu.pop_stack();
                    mmu.set_hl(value);
                }
                0xE2 => mmu.wb(0xFF00 + c as u16, a),
                0xE5 => mmu.push_stack(hl),
                0xE6 => {
                    let d8 = mmu.get_next_byte();
                    alu_and(mmu, d8);
                }
                0xE9 => mmu.pc = hl,
                0xEA => {
                    let d8 = mmu.get_next_word();
                    mmu.wb(d8, a)
                }
                0xEF => {
                    mmu.push_stack(mmu.pc);
                    mmu.pc = 0x0028;
                }
                0xF0 => {
                    let addr = 0xFF00 + (mmu.get_next_byte() as u16);
                    mmu.a = mmu.rb(addr);
                }
                0xF1 => {
                    let addr = mmu.pop_stack();
                    mmu.set_af(addr);
                }
                0xF3 => {
                    // Changes to IME are not instant, they happen _after_ the _next_ opcode.
                    mmu.interrupts.disable_ime();
                }
                0xF5 => mmu.push_stack(af),
                0xF6 => {
                    let value = mmu.get_next_byte();
                    alu_or(mmu, value);
                }
                0xFA => {
                    let address = mmu.get_next_word();
                    mmu.a = mmu.rb(address);
                }
                0xFB => {
                    // Changes to IME are not instant, they happen _after_ the _next_ opcode.
                    mmu.interrupts.enable_ime(2);
                }
                0xFE => {
                    let d8 = mmu.get_next_byte();
                    alu_cp(mmu, d8)
                }
                _ => self.panic_opcode(opcode, is_cbprefix, op_address),
            }
        } else {
            match opcode {
                0x11 => mmu.c = alu_rl(mmu, c),
                0x27 => mmu.a = alu_sla(mmu, a),
                0x30 => mmu.b = alu_swap(mmu, b),
                0x31 => mmu.c = alu_swap(mmu, c),
                0x32 => mmu.d = alu_swap(mmu, d),
                0x33 => mmu.e = alu_swap(mmu, e),
                0x34 => mmu.h = alu_swap(mmu, h),
                0x35 => mmu.l = alu_swap(mmu, l),
                0x36 => {
                    let value = alu_swap(mmu, mmu.rb(hl));
                    mmu.wb(hl, value);
                }
                0x37 => mmu.a = alu_swap(mmu, a),
                0x3F => mmu.a = alu_srl(mmu, a),
                0x40 => alu_bit(mmu, 0, b),
                0x41 => alu_bit(mmu, 0, c),
                0x42 => alu_bit(mmu, 0, d),
                0x43 => alu_bit(mmu, 0, e),
                0x44 => alu_bit(mmu, 0, h),
                0x45 => alu_bit(mmu, 0, l),
                0x47 => alu_bit(mmu, 0, a),
                0x50 => alu_bit(mmu, 2, b),
                0x51 => alu_bit(mmu, 2, c),
                0x52 => alu_bit(mmu, 2, d),
                0x53 => alu_bit(mmu, 2, e),
                0x54 => alu_bit(mmu, 2, h),
                0x55 => alu_bit(mmu, 2, l),
                0x58 => alu_bit(mmu, 3, b),
                0x59 => alu_bit(mmu, 3, c),
                0x5A => alu_bit(mmu, 3, d),
                0x5B => alu_bit(mmu, 3, e),
                0x5C => alu_bit(mmu, 3, h),
                0x5D => alu_bit(mmu, 3, l),
                0x60 => alu_bit(mmu, 4, b),
                0x61 => alu_bit(mmu, 4, c),
                0x62 => alu_bit(mmu, 4, d),
                0x63 => alu_bit(mmu, 4, e),
                0x64 => alu_bit(mmu, 4, h),
                0x65 => alu_bit(mmu, 4, l),
                0x68 => alu_bit(mmu, 5, b),
                0x69 => alu_bit(mmu, 5, c),
                0x6A => alu_bit(mmu, 5, d),
                0x6B => alu_bit(mmu, 5, e),
                0x6C => alu_bit(mmu, 5, h),
                0x6D => alu_bit(mmu, 5, l),
                0x5F => alu_bit(mmu, 3, a),
                0x78 => alu_bit(mmu, 7, b),
                0x79 => alu_bit(mmu, 7, c),
                0x7A => alu_bit(mmu, 7, d),
                0x7B => alu_bit(mmu, 7, e),
                0x7C => alu_bit(mmu, 7, h),
                0x7D => alu_bit(mmu, 7, l),
                0x7E => alu_bit(mmu, 7, mmu.rb(hl)),
                0x7F => alu_bit(mmu, 7, a),
                0x80 => mmu.b = alu_res(0, b),
                0x81 => mmu.b = alu_res(0, c),
                0x82 => mmu.b = alu_res(0, d),
                0x83 => mmu.b = alu_res(0, e),
                0x84 => mmu.b = alu_res(0, h),
                0x85 => mmu.b = alu_res(0, l),
                0x86 => mmu.wb(hl, alu_res(0, mmu.rb(hl))),
                0x87 => mmu.a = alu_res(0, a),
                0x88 => mmu.b = alu_res(1, b),
                0x89 => mmu.b = alu_res(1, c),
                0x8A => mmu.b = alu_res(1, d),
                0x8B => mmu.b = alu_res(1, e),
                0x8C => mmu.b = alu_res(1, h),
                0x8D => mmu.b = alu_res(1, l),
                0x90 => mmu.b = alu_res(2, b),
                0x91 => mmu.b = alu_res(2, c),
                0x92 => mmu.b = alu_res(2, d),
                0x93 => mmu.b = alu_res(2, e),
                0x94 => mmu.b = alu_res(2, h),
                0x95 => mmu.b = alu_res(2, l),
                0x98 => mmu.b = alu_res(3, b),
                0x99 => mmu.b = alu_res(3, c),
                0x9A => mmu.b = alu_res(3, d),
                0x9B => mmu.b = alu_res(3, e),
                0x9C => mmu.b = alu_res(3, h),
                0x9D => mmu.b = alu_res(3, l),
                0xA0 => mmu.b = alu_res(4, b),
                0xA1 => mmu.b = alu_res(4, c),
                0xA2 => mmu.b = alu_res(4, d),
                0xA3 => mmu.b = alu_res(4, e),
                0xA4 => mmu.b = alu_res(4, h),
                0xA5 => mmu.b = alu_res(4, l),
                0xA8 => mmu.b = alu_res(5, b),
                0xA9 => mmu.b = alu_res(5, c),
                0xAA => mmu.b = alu_res(5, d),
                0xAB => mmu.b = alu_res(5, e),
                0xAC => mmu.b = alu_res(5, h),
                0xAD => mmu.b = alu_res(5, l),
                0xB0 => mmu.b = alu_res(6, b),
                0xB1 => mmu.b = alu_res(6, c),
                0xB2 => mmu.b = alu_res(6, d),
                0xB3 => mmu.b = alu_res(6, e),
                0xB4 => mmu.b = alu_res(6, h),
                0xB5 => mmu.b = alu_res(6, l),
                0xB8 => mmu.b = alu_res(7, b),
                0xB9 => mmu.b = alu_res(7, c),
                0xBA => mmu.b = alu_res(7, d),
                0xBB => mmu.b = alu_res(7, e),
                0xBC => mmu.b = alu_res(7, h),
                0xBD => mmu.b = alu_res(7, l),

                _ => self.panic_opcode(opcode, is_cbprefix, op_address),
            }
        }

        // Change cycles to be the larger value as the action was taken, which is more expensive.
        // Only some operations are branching conditions with differing cycle lengths.
        if condition_met {
            cycles = self.opcodes.get_cycles(opcode, is_cbprefix, true);
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

        panic!("{}", msg);
    }
}
