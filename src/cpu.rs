use super::opcode::OpCodes;
use super::register::Register;
use super::MMU;
pub struct CPU {
    pc: u16,
    sp: u16,
    mmu: MMU,
    reg: Register,
    opcode_metadata: OpCodes,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            pc: 0,
            sp: 0, // TODO: this is not right.
            mmu: MMU::new(),
            reg: Register::new(),
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

        // Match an opcode and manipulate memory accordingly.
        match opcode {
            0x31 => self.sp = self.get_word(),
            _ => panic!(
                "Opcode: {} not handled.",
                self.opcode_metadata.get_opcode_repr(opcode, is_cbprefix)
            ),
        };
    }

    fn alu_xor(&mut self, n: u8) {
        self.reg.a ^= n;
    }

    fn get_byte(&mut self) -> u8 {
        let byte = self.mmu.rb(self.pc);
        self.pc += 1;
        byte
    }

    fn get_word(&mut self) -> u16 {
        let word = self.mmu.rw(self.pc);
        self.pc += 2;
        word
    }
}
