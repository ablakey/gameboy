use super::opcode::OpCodes;
use super::registers::Registers;
use super::MMU;
pub struct CPU {
    pc: u16,
    sp: u16,
    mmu: MMU,
    reg: Registers,
    opcodes: OpCodes,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            pc: 0,
            sp: 0, // TODO: this is not right.
            mmu: MMU::new(),
            reg: Registers::new(),
            opcodes: OpCodes::from_path("data/opcodes.json").unwrap(),
        }
    }

    /// Perform a single opcode step and return how many cycles that took.
    /// Return the number of m-cycles required to perform the operation. This will be used for
    /// regulating how fast the CPU is emulated at.
    pub fn step(&mut self) -> u8 {
        let mut opcode = self.get_byte();
        let is_cbprefix = opcode == 0xCB;

        // If the byte is not the opcode but actually the prefix, get another byte.
        if is_cbprefix {
            opcode = self.get_byte();
        }

        println!("{}", self.opcodes.get_opcode_repr(opcode, is_cbprefix));

        // The number of m-cycles required for this operation. This may be updated by an operation
        // if a conditional branch was NOT performed that costs less. We assume the condition is not
        // met.
        let mut cycles = self.opcodes.get_cycles(opcode, is_cbprefix, false);
        let mut condition_met = false;

        // Match an opcode and manipulate memory accordingly.
        if !is_cbprefix {
            match opcode {
                0x0C => self.reg.c += 1,
                0x0E => self.reg.c = self.get_byte(),
                0x3E => self.reg.a = self.get_byte(),
                0x20 => {
                    if !self.reg.flag_z() {
                        self.pc = self.pc.wrapping_add(self.get_signed_byte() as u16);
                        condition_met = true;
                    }
                }
                0x21 => {
                    let b = self.get_word();
                    self.reg.set_hl(b)
                }
                0x31 => self.sp = self.get_word(),
                0x32 => {
                    self.mmu.write(self.reg.hl(), self.reg.a); // Set (HL) to A.
                    self.reg.set_hl(self.reg.hl().wrapping_sub(1)); // Decrement.
                }
                0x77 => self.mmu.write(self.reg.hl(), self.reg.a),
                0x7C => self.reg.a = self.reg.h,
                0x9F => self.reg.alu_sbc(self.reg.a),
                0xAF => self.reg.alu_xor(self.reg.a),
                0xE2 => self.mmu.write(0xFF00 + self.reg.c as u16, self.reg.a),

                _ => panic!(
                    "Opcode: {} not handled.",
                    self.opcodes.get_opcode_repr(opcode, is_cbprefix)
                ),
            }
        } else {
            match opcode {
                0x7C => self.reg.alu_bit(self.reg.h, 7),
                _ => panic!(
                    "CBPREFIX Opcode: {} not handled.",
                    self.opcodes.get_opcode_repr(opcode, is_cbprefix)
                ),
            }
        }

        // Change cycles to be the smaller value (action not taken).
        if condition_met {
            cycles = self.opcodes.get_cycles(opcode, is_cbprefix, false);
        }

        cycles
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
}
