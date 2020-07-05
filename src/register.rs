/// Generate getters and setters for register pairs. 8-bit registers can be combined into pairs to
/// act as 16-bit registers. There are four to be created: AF, BC, DE, HL.
macro_rules! create_word_accessors {
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

pub struct Register {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    f: u8,
}

impl Register {
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

    create_word_accessors!(af, set_af, a, f);
    create_word_accessors!(bc, set_bc, b, c);
    create_word_accessors!(de, set_de, d, e);
    create_word_accessors!(hl, set_hl, h, l);
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test getters and setters for the word registers. Read above for details on these 16-bit
    // registers backed by two 8-bit registers.
    macro_rules! test_word_registers {
        ($getname:ident, $setname:ident, $reg1:ident, $reg2:ident) => {
            #[test]
            fn $getname() {
                let mut reg = Register::new();
                reg.$reg1 = 0xFF;
                reg.$reg2 = 0x11;
                assert_eq!(reg.$getname(), 0xFF11)
            }

            #[test]
            fn $setname() {
                let mut reg = Register::new();
                reg.$setname(0xFF11);
                assert_eq!(reg.$reg1, 0xFF);
                assert_eq!(reg.$reg2, 0x11);
            }
        };
    }

    test_word_registers!(af, set_af, a, f);
    test_word_registers!(bc, set_bc, b, c);
    test_word_registers!(de, set_de, d, e);
    test_word_registers!(hl, set_hl, h, l);
}
