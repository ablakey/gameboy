use super::opcode::OpCodes;

use super::alu;
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
            sp,
            ..
        } = *mmu;

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
                0x04 => mmu.b = alu::inc(mmu, b),
                0x05 => mmu.b = alu::dec(mmu, b),
                0x06 => mmu.b = mmu.get_next_byte(),
                0x07 => {
                    mmu.a = alu::rlc(mmu, a); // RLCA is almost the same as RLC but Z is always 0.
                    mmu.set_flag_z(false);
                }
                0x09 => alu::add_16(mmu, bc),
                0x0A => mmu.a = mmu.rb(bc),
                0x0B => mmu.set_bc(bc.wrapping_sub(1)),
                0x0C => mmu.c += 1,
                0x0D => mmu.c = alu::dec(mmu, c),
                0x0E => mmu.c = mmu.get_next_byte(),
                0x11 => {
                    let d16 = mmu.get_next_word();
                    mmu.set_de(d16);
                }
                0x12 => mmu.wb(de, a),
                0x13 => mmu.set_de(de.wrapping_add(1)),
                0x15 => mmu.d = alu::dec(mmu, d),
                0x16 => mmu.d = mmu.get_next_byte(),
                0x17 => {
                    // RLA is same as RL A but Z flag is unset.
                    mmu.a = alu::rl(mmu, a);
                    mmu.set_flag_z(false);
                }
                0x18 => {
                    let r8 = mmu.get_signed_byte(); // Must get first as it mutates PC.
                    mmu.pc = mmu.pc.wrapping_add(r8 as u16);
                }
                0x19 => alu::add_16(mmu, de),
                0x1A => mmu.a = mmu.rb(de),
                0x1B => mmu.set_de(de.wrapping_sub(1)),
                0x1C => mmu.e = alu::inc(mmu, e),
                0x1D => mmu.e = alu::dec(mmu, e),
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
                0x24 => mmu.h = alu::inc(mmu, h),
                0x25 => mmu.h = alu::dec(mmu, h),
                0x26 => mmu.h = mmu.get_next_byte(),
                0x27 => alu::daa(mmu),
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
                0x2B => mmu.set_hl(hl.wrapping_sub(1)),
                0x2C => mmu.l = alu::inc(mmu, l),
                0x2D => mmu.l = alu::dec(mmu, l),
                0x2E => mmu.l = mmu.get_next_byte(),
                0x2F => alu::cpl(mmu),
                0x30 => {
                    let r8 = mmu.get_signed_byte(); // Need to get byte to inc PC either way.
                    if !mmu.flag_c() {
                        mmu.pc = mmu.pc.wrapping_add(r8 as u16);
                        condition_met = true;
                    }
                }
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
                    let value = alu::inc(mmu, mmu.rb(hl));
                    mmu.wb(hl, value);
                }
                0x35 => {
                    let value = alu::dec(mmu, mmu.rb(hl));
                    mmu.wb(hl, value);
                }
                0x36 => {
                    let d8 = mmu.get_next_byte();
                    mmu.wb(hl, d8);
                }
                0x38 => {
                    let r8 = mmu.get_signed_byte();
                    if mmu.flag_c() {
                        mmu.pc.wrapping_add(r8 as u16);
                        condition_met = true;
                    }
                }
                0x3A => {
                    mmu.a = mmu.rb(hl);
                    mmu.set_hl(hl.wrapping_sub(1));
                }
                0x3B => mmu.sp = sp.wrapping_sub(1),
                0x3C => mmu.a = alu::inc(mmu, a),
                0x3D => mmu.a = alu::dec(mmu, a),
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
                0x80 => alu::add(mmu, b),
                0x81 => alu::add(mmu, c),
                0x82 => alu::add(mmu, d),
                0x83 => alu::add(mmu, e),
                0x84 => alu::add(mmu, h),
                0x85 => alu::add(mmu, l),
                0x86 => alu::add(mmu, mmu.rb(hl)),
                0x87 => alu::add(mmu, a),
                0x88 => alu::adc(mmu, b),
                0x89 => alu::adc(mmu, c),
                0x8A => alu::adc(mmu, d),
                0x8B => alu::adc(mmu, e),
                0x8C => alu::adc(mmu, h),
                0x8D => alu::adc(mmu, l),
                0x8E => alu::adc(mmu, mmu.rb(hl)),
                0x8F => alu::adc(mmu, a),
                0x90 => alu::sub(mmu, b),
                0x91 => alu::sub(mmu, c),
                0x92 => alu::sub(mmu, d),
                0x93 => alu::sub(mmu, e),
                0x94 => alu::sub(mmu, h),
                0x95 => alu::sub(mmu, l),
                0x96 => alu::sub(mmu, mmu.rb(hl)),
                0x97 => alu::sub(mmu, a),
                0x98 => alu::sbc(mmu, b),
                0x99 => alu::sbc(mmu, c),
                0x9A => alu::sbc(mmu, d),
                0x9B => alu::sbc(mmu, e),
                0x9C => alu::sbc(mmu, h),
                0x9D => alu::sbc(mmu, l),
                0x9E => alu::sbc(mmu, mmu.rb(hl)),
                0x9F => alu::sbc(mmu, a),
                0xA1 => alu::and(mmu, c),
                0xA7 => alu::and(mmu, a),
                0xA8 => alu::xor(mmu, b),
                0xA9 => alu::xor(mmu, c),
                0xAA => alu::xor(mmu, d),
                0xAB => alu::xor(mmu, e),
                0xAC => alu::xor(mmu, h),
                0xAD => alu::xor(mmu, l),
                0xAE => alu::xor(mmu, mmu.rb(hl)),
                0xAF => alu::xor(mmu, a),
                0xB0 => alu::or(mmu, b),
                0xB1 => alu::or(mmu, c),
                0xB2 => alu::or(mmu, d),
                0xB3 => alu::or(mmu, e),
                0xB4 => alu::or(mmu, h),
                0xB5 => alu::or(mmu, l),
                0xB6 => alu::or(mmu, mmu.rb(hl)),
                0xB7 => alu::or(mmu, a),
                0xB8 => alu::cp(mmu, b),
                0xB9 => alu::cp(mmu, c),
                0xBA => alu::cp(mmu, d),
                0xBB => alu::cp(mmu, e),
                0xBC => alu::cp(mmu, h),
                0xBD => alu::cp(mmu, l),
                0xBE => alu::cp(mmu, mmu.rb(hl)),
                0xBF => alu::cp(mmu, a),
                0xC0 => {
                    if !mmu.flag_z() {
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
                    if !mmu.flag_z() {
                        mmu.pc = address;
                        condition_met = true;
                    }
                }
                0xC3 => mmu.pc = mmu.get_next_word(),
                0xC5 => mmu.push_stack(bc),
                0xC6 => {
                    let value = mmu.get_next_byte();
                    alu::add(mmu, value);
                }
                0xC8 => {
                    if mmu.flag_z() {
                        mmu.pc = mmu.pop_stack();
                        condition_met = true;
                    }
                }
                0xC9 => mmu.pc = mmu.pop_stack(),
                0xCA => {
                    let address = mmu.get_next_word(); // Need to get regardless to advance PC.
                    if mmu.flag_z() {
                        mmu.pc = address;
                        condition_met = true;
                    }
                }
                0xCD => {
                    let a16 = mmu.get_next_word(); // Advances mmu.pc to the next instruction.
                    mmu.push_stack(mmu.pc); // mmu.pc is the next instruction to be run.
                    mmu.pc = a16;
                }
                0xCE => {
                    let value = mmu.get_next_byte();
                    alu::adc(mmu, value);
                }
                0xD0 => {
                    if !mmu.flag_c() {
                        mmu.pc = mmu.pop_stack();
                        condition_met = true;
                    }
                }
                0xD1 => {
                    let value = mmu.pop_stack();
                    mmu.set_de(value);
                }

                0xD5 => mmu.push_stack(de),
                0xD6 => {
                    let value = mmu.get_next_byte();
                    alu::sub(mmu, value);
                }
                0xD8 => {
                    if mmu.flag_c() {
                        mmu.pc = mmu.pop_stack();
                        condition_met = true;
                    }
                }
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
                    alu::and(mmu, d8);
                }
                0xE9 => mmu.pc = hl,
                0xEA => {
                    let d8 = mmu.get_next_word();
                    mmu.wb(d8, a)
                }
                0xEE => {
                    let value = mmu.get_next_byte();
                    alu::xor(mmu, value);
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
                    alu::or(mmu, value);
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
                    alu::cp(mmu, d8)
                }
                _ => self.panic_opcode(opcode, is_cbprefix, op_address),
            }
        } else {
            match opcode {
                0x11 => mmu.c = alu::rl(mmu, c),
                0x27 => mmu.a = alu::sla(mmu, a),
                0x30 => mmu.b = alu::swap(mmu, b),
                0x31 => mmu.c = alu::swap(mmu, c),
                0x32 => mmu.d = alu::swap(mmu, d),
                0x33 => mmu.e = alu::swap(mmu, e),
                0x34 => mmu.h = alu::swap(mmu, h),
                0x35 => mmu.l = alu::swap(mmu, l),
                0x36 => {
                    let value = alu::swap(mmu, mmu.rb(hl));
                    mmu.wb(hl, value);
                }
                0x37 => mmu.a = alu::swap(mmu, a),
                0x3F => mmu.a = alu::srl(mmu, a),
                0x40 => alu::bit(mmu, 0, b),
                0x41 => alu::bit(mmu, 0, c),
                0x42 => alu::bit(mmu, 0, d),
                0x43 => alu::bit(mmu, 0, e),
                0x44 => alu::bit(mmu, 0, h),
                0x45 => alu::bit(mmu, 0, l),
                0x46 => alu::bit(mmu, 0, mmu.rb(hl)),
                0x47 => alu::bit(mmu, 0, a),
                0x48 => alu::bit(mmu, 1, b),
                0x49 => alu::bit(mmu, 1, c),
                0x4A => alu::bit(mmu, 1, d),
                0x4B => alu::bit(mmu, 1, e),
                0x4C => alu::bit(mmu, 1, h),
                0x4D => alu::bit(mmu, 1, l),
                0x4E => alu::bit(mmu, 1, mmu.rb(hl)),
                0x4F => alu::bit(mmu, 1, a),
                0x50 => alu::bit(mmu, 2, b),
                0x51 => alu::bit(mmu, 2, c),
                0x52 => alu::bit(mmu, 2, d),
                0x53 => alu::bit(mmu, 2, e),
                0x54 => alu::bit(mmu, 2, h),
                0x55 => alu::bit(mmu, 2, l),
                0x56 => alu::bit(mmu, 2, mmu.rb(hl)),
                0x57 => alu::bit(mmu, 2, a),
                0x58 => alu::bit(mmu, 3, b),
                0x59 => alu::bit(mmu, 3, c),
                0x5A => alu::bit(mmu, 3, d),
                0x5B => alu::bit(mmu, 3, e),
                0x5C => alu::bit(mmu, 3, h),
                0x5D => alu::bit(mmu, 3, l),
                0x5E => alu::bit(mmu, 3, mmu.rb(hl)),
                0x5F => alu::bit(mmu, 3, a),
                0x60 => alu::bit(mmu, 4, b),
                0x61 => alu::bit(mmu, 4, c),
                0x62 => alu::bit(mmu, 4, d),
                0x63 => alu::bit(mmu, 4, e),
                0x64 => alu::bit(mmu, 4, h),
                0x65 => alu::bit(mmu, 4, l),
                0x66 => alu::bit(mmu, 4, mmu.rb(hl)),
                0x67 => alu::bit(mmu, 4, a),
                0x68 => alu::bit(mmu, 5, b),
                0x69 => alu::bit(mmu, 5, c),
                0x6A => alu::bit(mmu, 5, d),
                0x6B => alu::bit(mmu, 5, e),
                0x6C => alu::bit(mmu, 5, h),
                0x6D => alu::bit(mmu, 5, l),
                0x6E => alu::bit(mmu, 5, mmu.rb(hl)),
                0x6F => alu::bit(mmu, 5, a),
                0x70 => alu::bit(mmu, 6, b),
                0x71 => alu::bit(mmu, 6, c),
                0x72 => alu::bit(mmu, 6, d),
                0x73 => alu::bit(mmu, 6, e),
                0x74 => alu::bit(mmu, 6, h),
                0x75 => alu::bit(mmu, 6, l),
                0x76 => alu::bit(mmu, 6, mmu.rb(hl)),
                0x77 => alu::bit(mmu, 6, a),
                0x78 => alu::bit(mmu, 7, b),
                0x79 => alu::bit(mmu, 7, c),
                0x7A => alu::bit(mmu, 7, d),
                0x7B => alu::bit(mmu, 7, e),
                0x7C => alu::bit(mmu, 7, h),
                0x7D => alu::bit(mmu, 7, l),
                0x7E => alu::bit(mmu, 7, mmu.rb(hl)),
                0x7F => alu::bit(mmu, 7, a),
                0x80 => mmu.b = alu::res(0, b),
                0x81 => mmu.c = alu::res(0, c),
                0x82 => mmu.d = alu::res(0, d),
                0x83 => mmu.e = alu::res(0, e),
                0x84 => mmu.h = alu::res(0, h),
                0x85 => mmu.l = alu::res(0, l),
                0x86 => mmu.wb(hl, alu::res(0, mmu.rb(hl))),
                0x87 => mmu.a = alu::res(0, a),
                0x88 => mmu.b = alu::res(1, b),
                0x89 => mmu.c = alu::res(1, c),
                0x8A => mmu.d = alu::res(1, d),
                0x8B => mmu.e = alu::res(1, e),
                0x8C => mmu.h = alu::res(1, h),
                0x8D => mmu.l = alu::res(1, l),
                0x8E => mmu.wb(hl, alu::res(1, mmu.rb(hl))),
                0x8F => mmu.a = alu::res(1, a),
                0x90 => mmu.b = alu::res(2, b),
                0x91 => mmu.c = alu::res(2, c),
                0x92 => mmu.d = alu::res(2, d),
                0x93 => mmu.e = alu::res(2, e),
                0x94 => mmu.h = alu::res(2, h),
                0x95 => mmu.l = alu::res(2, l),
                0x96 => mmu.wb(hl, alu::res(2, mmu.rb(hl))),
                0x97 => mmu.a = alu::res(2, a),
                0x98 => mmu.b = alu::res(3, b),
                0x99 => mmu.c = alu::res(3, c),
                0x9A => mmu.d = alu::res(3, d),
                0x9B => mmu.e = alu::res(3, e),
                0x9C => mmu.h = alu::res(3, h),
                0x9D => mmu.l = alu::res(3, l),
                0x9E => mmu.wb(hl, alu::res(3, mmu.rb(hl))),
                0x9F => mmu.a = alu::res(3, a),
                0xA0 => mmu.b = alu::res(4, b),
                0xA1 => mmu.c = alu::res(4, c),
                0xA2 => mmu.d = alu::res(4, d),
                0xA3 => mmu.e = alu::res(4, e),
                0xA4 => mmu.h = alu::res(4, h),
                0xA5 => mmu.l = alu::res(4, l),
                0xA6 => mmu.wb(hl, alu::res(4, mmu.rb(hl))),
                0xA7 => mmu.a = alu::res(4, a),
                0xA8 => mmu.b = alu::res(5, b),
                0xA9 => mmu.c = alu::res(5, c),
                0xAA => mmu.d = alu::res(5, d),
                0xAB => mmu.e = alu::res(5, e),
                0xAC => mmu.h = alu::res(5, h),
                0xAD => mmu.l = alu::res(5, l),
                0xAE => mmu.wb(hl, alu::res(5, mmu.rb(hl))),
                0xAF => mmu.a = alu::res(5, a),
                0xB0 => mmu.b = alu::res(6, b),
                0xB1 => mmu.c = alu::res(6, c),
                0xB2 => mmu.d = alu::res(6, d),
                0xB3 => mmu.e = alu::res(6, e),
                0xB4 => mmu.h = alu::res(6, h),
                0xB5 => mmu.l = alu::res(6, l),
                0xB6 => mmu.wb(hl, alu::res(6, mmu.rb(hl))),
                0xB7 => mmu.a = alu::res(6, a),
                0xB8 => mmu.b = alu::res(7, b),
                0xB9 => mmu.c = alu::res(7, c),
                0xBA => mmu.d = alu::res(7, d),
                0xBB => mmu.e = alu::res(7, e),
                0xBC => mmu.h = alu::res(7, h),
                0xBD => mmu.l = alu::res(7, l),
                0xBE => mmu.wb(hl, alu::res(7, mmu.rb(hl))),
                0xBF => mmu.a = alu::res(7, a),
                0xC0 => mmu.b = alu::set(0, b),
                0xC1 => mmu.c = alu::set(0, c),
                0xC2 => mmu.d = alu::set(0, d),
                0xC3 => mmu.e = alu::set(0, e),
                0xC4 => mmu.h = alu::set(0, h),
                0xC5 => mmu.l = alu::set(0, l),
                0xC6 => mmu.wb(hl, alu::set(0, mmu.rb(hl))),
                0xC7 => mmu.a = alu::set(0, a),
                0xC8 => mmu.b = alu::set(1, b),
                0xC9 => mmu.c = alu::set(1, c),
                0xCA => mmu.d = alu::set(1, d),
                0xCB => mmu.e = alu::set(1, e),
                0xCC => mmu.h = alu::set(1, h),
                0xCD => mmu.l = alu::set(1, l),
                0xCE => mmu.wb(hl, alu::set(1, mmu.rb(hl))),
                0xCF => mmu.a = alu::set(1, a),
                0xD0 => mmu.b = alu::set(2, b),
                0xD1 => mmu.c = alu::set(2, c),
                0xD2 => mmu.d = alu::set(2, d),
                0xD3 => mmu.e = alu::set(2, e),
                0xD4 => mmu.h = alu::set(2, h),
                0xD5 => mmu.l = alu::set(2, l),
                0xD6 => mmu.wb(hl, alu::set(2, mmu.rb(hl))),
                0xD7 => mmu.a = alu::set(2, a),
                0xD8 => mmu.b = alu::set(3, b),
                0xD9 => mmu.c = alu::set(3, c),
                0xDA => mmu.d = alu::set(3, d),
                0xDB => mmu.e = alu::set(3, e),
                0xDC => mmu.h = alu::set(3, h),
                0xDD => mmu.l = alu::set(3, l),
                0xDE => mmu.wb(hl, alu::set(3, mmu.rb(hl))),
                0xDF => mmu.a = alu::set(3, a),
                0xE0 => mmu.b = alu::set(4, b),
                0xE1 => mmu.c = alu::set(4, c),
                0xE2 => mmu.d = alu::set(4, d),
                0xE3 => mmu.e = alu::set(4, e),
                0xE4 => mmu.h = alu::set(4, h),
                0xE5 => mmu.l = alu::set(4, l),
                0xE6 => mmu.wb(hl, alu::set(4, mmu.rb(hl))),
                0xE7 => mmu.a = alu::set(4, a),
                0xE8 => mmu.b = alu::set(5, b),
                0xE9 => mmu.c = alu::set(5, c),
                0xEA => mmu.d = alu::set(5, d),
                0xEB => mmu.e = alu::set(5, e),
                0xEC => mmu.h = alu::set(5, h),
                0xED => mmu.l = alu::set(5, l),
                0xEE => mmu.wb(hl, alu::set(5, mmu.rb(hl))),
                0xEF => mmu.a = alu::set(5, a),
                0xF0 => mmu.b = alu::set(6, b),
                0xF1 => mmu.c = alu::set(6, c),
                0xF2 => mmu.d = alu::set(6, d),
                0xF3 => mmu.e = alu::set(6, e),
                0xF4 => mmu.h = alu::set(6, h),
                0xF5 => mmu.l = alu::set(6, l),
                0xF6 => mmu.wb(hl, alu::set(6, mmu.rb(hl))),
                0xF7 => mmu.a = alu::set(6, a),
                0xF8 => mmu.b = alu::set(7, b),
                0xF9 => mmu.c = alu::set(7, c),
                0xFA => mmu.d = alu::set(7, d),
                0xFB => mmu.e = alu::set(7, e),
                0xFC => mmu.h = alu::set(7, h),
                0xFD => mmu.l = alu::set(7, l),
                0xFE => mmu.wb(hl, alu::set(7, mmu.rb(hl))),
                0xFF => mmu.a = alu::set(7, a),
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

    /// Step the emulation forward one unit. A unit can be a different length in cycles depending
    /// on what is done. Generally this is three things:
    /// 1. Perform an opcode instruction.
    /// 2. Handle an interrupt, jumping to an interrupt address.
    /// 3. Do nothing because the CPU is halted.
    pub fn step(&self, mmu: &mut MMU) -> u8 {
        // If EI or DI was called, tick down the delay and possibly modify IME.
        mmu.interrupts.tick_ime_timer();

        // Try to handle an interrupt. If none was handled, try to do an opcode if not halted.
        match mmu.try_interrupt() {
            0 => {
                if mmu.interrupts.is_halted {
                    1
                } else {
                    self.do_opcode(mmu)
                }
            }
            n => n,
        }
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
