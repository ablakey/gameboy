/// Generate getters and setters for register pairs. 8-bit registers can be combined into pairs to
/// act as 16-bit registers. There are four to be created: AF, BC, DE, HL.
macro_rules! create_register_pair {
    ($getname:ident, $setname:ident, $reg_1:ident, $reg_2:ident) => {
        pub fn $getname(&self) -> u16 {
            ((self.$reg_1 as u16) << 8) | (self.$reg_2 as u16)
        }

        pub fn $setname(&mut self, value: u16) {
            self.$reg_1 = (value >> 8) as u8;
            self.$reg_2 = value as u8;
        }
    };
}

macro_rules! create_flag {
    ($getter:ident, $setter:ident, $mask:expr) => {
        pub fn $getter(&self) -> bool {
            self.f & $mask != 0
        }

        pub fn $setter(&mut self, value: bool) {
            if value {
                self.f |= $mask;
            } else {
                self.f &= !$mask;
            }
        }
    };
}

pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    f: u8,
}

impl Registers {
    pub fn new() -> Self {
        Self {
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

    create_flag!(flag_z, set_flag_z, 0b10000000);
    create_flag!(flag_n, set_flag_n, 0b01000000);
    create_flag!(flag_h, set_flag_h, 0b00100000);
    create_flag!(flag_c, set_flag_c, 0b00010000);

    create_register_pair!(af, set_af, a, f);
    create_register_pair!(bc, set_bc, b, c);
    create_register_pair!(de, set_de, d, e);
    create_register_pair!(hl, set_hl, h, l);

    /// Logical exclusive OR n with register A, result in A.
    pub fn alu_xor(&mut self, n: u8) {
        self.a ^= n;
        self.set_flag_z(self.a == 0);
        self.set_flag_h(false);
        self.set_flag_n(false);
        self.set_flag_c(false);
    }

    /// Increment register value. Set Z if zero, H if half carry (bit 3), N reset.
    pub fn alu_inc(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_add(1);

        // Calculate a half-carry by isolating the low nibble, adding one, and seeing if the result
        // is larger than 0xF (fourth bit is high).
        println!("{}", new_value);
        self.set_flag_h(((0xF & value) + 1) > 0xF);
        self.set_flag_z(new_value == 0);
        self.set_flag_n(false);

        new_value
    }

    /// Test a specific bit of a given byte. If it is unset, set the Z flag high.
    pub fn alu_bit(&mut self, value: u8, bit_num: u8) {
        let mask = 1 << bit_num;
        let is_unset = value & mask == 0;
        self.set_flag_z(is_unset);
        self.set_flag_n(false);
        self.set_flag_h(true);
    }

    /// Subtract value from A.
    /// H is set if a half borrow occurs. This is calculated by isolating just the bottom nibble
    /// and calculating a full borrow of that. This is done by seeing if the operand is greater than
    /// self.a, because that means there would be a wrap around (aka a borrow happens).
    /// C is set if there is a full borrow. Same method for detecting: is the operand larger?
    pub fn alu_sub(&mut self, value: u8) {
        let new_a = self.a.wrapping_sub(value);
        self.set_flag_z(new_a == 0);
        self.set_flag_n(true);
        self.set_flag_h((self.a & 0x0F) < (value & 0x0F));
        self.set_flag_c(self.a < value);
        self.a = new_a;
    }

    /// Subtract value and the carry bit from A.
    pub fn alu_sbc(&mut self, value: u8) {
        self.alu_sub(value + self.flag_c() as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test setting the af register. Given each register is implemented using a macro, we only need
    /// to test one of them.
    #[test]
    fn test_af() {
        let mut reg = Registers::new();
        reg.a = 0xFF;
        reg.f = 0x11;
        assert_eq!(reg.af(), 0xFF11)
    }

    /// Test getting the af register. Given each register is implemented using a macro, we only need
    /// to test one of them.
    #[test]
    fn test_set_af() {
        let mut reg = Registers::new();
        reg.set_af(0xFF11);
        assert_eq!(reg.a, 0xFF);
        assert_eq!(reg.f, 0x11);
    }

    #[test]
    fn test_get_flags() {
        let mut reg = Registers::new();
        reg.f = 0b10100000;
        assert_eq!(reg.flag_z(), true);
        assert_eq!(reg.flag_n(), false);
        assert_eq!(reg.flag_h(), true);
        assert_eq!(reg.flag_c(), false);
    }

    #[test]
    fn test_set_flags() {
        let mut reg = Registers::new();
        reg.set_flag_z(true);
        reg.set_flag_n(true);
        reg.set_flag_h(true);
        reg.set_flag_c(true);
        assert_eq!(reg.f, 0b11110000, "{:b}", reg.f);

        reg.set_flag_z(true);
        reg.set_flag_n(true);
        reg.set_flag_h(false);
        reg.set_flag_c(false);
        assert_eq!(reg.f, 0b11000000, "{:b}", reg.f);
    }

    #[test]
    fn test_alu_inc() {
        // Test zero flag, half-carry, and roll-over.
        let mut reg = Registers::new();
        reg.a = 0xFF;
        reg.a = reg.alu_inc(reg.a);
        assert_eq!(reg.a, 0x0);
        assert_eq!(reg.flag_z(), true);
        assert_eq!(reg.flag_h(), true);
    }

    #[test]
    fn test_alu_xor() {
        let mut reg = Registers::new();
        reg.alu_xor(0x11);
        assert_eq!(reg.a, 0x11);
        assert_eq!(reg.flag_z(), false);

        reg.alu_xor(0xFF);
        assert_eq!(reg.a, 0xEE);
        assert_eq!(reg.flag_z(), false);

        reg.a = 0x00;
        reg.alu_xor(0x00);
        assert_eq!(reg.a, 0x00);
        assert_eq!(reg.flag_z(), true);
    }

    #[test]
    fn test_alu_bit() {
        let mut reg = Registers::new();
        reg.a = 0b00001000;
        reg.alu_bit(reg.a, 3);
        assert_eq!(reg.flag_z(), false);

        reg.a = 0b00000000;
        reg.alu_bit(reg.a, 3);
        assert_eq!(reg.flag_z(), true);
    }

    #[test]
    fn test_alu_sub() {
        let mut reg = Registers::new();
        reg.a = 0x10;
        reg.alu_sub(0xFF);
        assert_eq!(reg.a, 0x11);
        assert_eq!(reg.flag_z(), false);
        assert_eq!(reg.flag_n(), true);
        assert_eq!(reg.flag_h(), true);
        assert_eq!(reg.flag_c(), true);
    }

    #[test]
    fn test_alu_sub_no_borrows() {
        let mut reg = Registers::new();
        reg.a = 0xFF;
        reg.alu_sub(0xFF);
        assert_eq!(reg.a, 0x00);
        assert_eq!(reg.flag_z(), true);
        assert_eq!(reg.flag_n(), true);
        assert_eq!(reg.flag_h(), false);
        assert_eq!(reg.flag_c(), false);
    }
}
