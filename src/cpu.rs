use super::opcode::OpCodes;
use super::registers::Registers;
use super::MMU;
pub struct CPU {
    pc: u16,
    sp: u16,
    mmu: MMU,
    reg: Registers,
    opcode_metadata: OpCodes,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            pc: 0,
            sp: 0, // TODO: this is not right.
            mmu: MMU::new(),
            reg: Registers::new(),
            opcode_metadata: OpCodes::from_path("data/opcodes.json").unwrap(),
        }
    }

    /// Perform a single opcode step and return how many cycles that took.
    pub fn step(&mut self) {
        let mut opcode = self.get_byte();
        let is_cbprefix = opcode == 0xCB;

        // If the byte is not the opcode but actually the prefix, get another byte.
        if is_cbprefix {
            opcode = self.get_byte();
        }

        println!(
            "{}",
            self.opcode_metadata.get_opcode_repr(opcode, is_cbprefix)
        );

        // Match an opcode and manipulate memory accordingly.
        match opcode {
            0x21 => {
                let b = self.get_word();
                self.reg.set_hl(b)
            }
            0x31 => self.sp = self.get_word(),
            0x32 => {
                let addr = self.reg.hl();
                let value = self.get_byte();
                self.mmu.write(addr, value);
            }
            0x7C => self.reg.a = self.reg.h,
            0xAF => self.alu_xor(self.reg.a),
            _ => panic!(
                "Opcode: {} not handled.",
                self.opcode_metadata.get_opcode_repr(opcode, is_cbprefix)
            ),
        };
    }

    /// Get the next byte in memory and advance the program counter by 1.
    fn get_byte(&mut self) -> u8 {
        let byte = self.mmu.read_byte(self.pc);
        self.pc += 1;
        byte
    }

    /// Get the next word in memory and advance the program counter by 2.
    fn get_word(&mut self) -> u16 {
        let word = self.mmu.read_word(self.pc);
        self.pc += 2;
        word
    }
}

/// Implement the ALU functions that drive math and boolean logic opcodes.
impl CPU {
    /// Logical exclusive OR n with register A, result in A.
    ///
    fn alu_xor(&mut self, n: u8) {
        self.reg.a ^= n;
        self.reg.set_flag(Flag::Z)
    }
}

mod tests {
    use super::*;

    #[test]
    fn it_xor_reg_a() {
        let mut cpu = CPU::new();
        cpu.alu_xor(0xFF); // 0x00 ^ 0xFF
        assert_eq!(cpu.reg.a, 0xFF);
        cpu.alu_xor(0x11); // 0xFF ^ 0x11
        assert_eq!(cpu.reg.a, 0xEE);
    }
}
