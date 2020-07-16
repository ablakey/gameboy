use super::MMU;
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
            self.f & (1 << $mask) != 0
        }

        pub fn $setter(&mut self, value: bool) {
            if value {
                self.f |= (1 << $mask);
            } else {
                self.f &= !(1 << $mask);
            }
        }
    };
}

impl MMU {
    create_flag!(flag_z, set_flag_z, 7);
    create_flag!(flag_n, set_flag_n, 6);
    create_flag!(flag_h, set_flag_h, 5);
    create_flag!(flag_c, set_flag_c, 4);

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
        let mut mmu = MMU::new();
        mmu.a = 0xFF;
        mmu.f = 0x11;
        assert_eq!(mmu.af(), 0xFF11)
    }

    /// Test getting the af register. Given each register is implemented using a macro, we only need
    /// to test one of them.
    #[test]
    fn test_set_af() {
        let mut mmu = MMU::new();
        mmu.set_af(0xFF11);
        assert_eq!(mmu.a, 0xFF);
        assert_eq!(mmu.f, 0x11);
    }

    #[test]
    fn test_get_flags() {
        let mmu = &mut MMU::new();
        mmu.f = 0b10100000;
        assert_eq!(mmu.flag_z(), true);
        assert_eq!(mmu.flag_h(), true);
    }

    #[test]
    fn test_set_flags() {
        let mut mmu = MMU::new();
        mmu.set_flag_z(true);
        mmu.set_flag_n(true);
        mmu.set_flag_h(true);
        mmu.set_flag_c(true);
        assert_eq!(mmu.f, 0b11110000, "{:b}", mmu.f);

        mmu.set_flag_z(true);
        mmu.set_flag_n(true);
        mmu.set_flag_h(false);
        mmu.set_flag_c(false);
        assert_eq!(mmu.f, 0b11000000, "{:b}", mmu.f);
    }
}
