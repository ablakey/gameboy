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
}
