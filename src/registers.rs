/// Generate getters and setters for register pairs. 8-bit registers can be combined into pairs to
/// act as 16-bit registers. There are four to be created: AF, BC, DE, HL.
macro_rules! create_pair {
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

/// Available registers, both single and pairs.
#[derive(Debug)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
}

/// CPU flags.
/// TODO: explain what each one means.
pub enum Flag {
    C,
    N,
    H,
    Z,
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

    pub fn get(&self, register: Register) -> u8 {
        match register {
            Register::A => self.a,
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            _ => panic!("Tried to access a non 8-bit register: {:?}", register),
        }
    }

    pub fn get_pair(&self, register: Register) -> u16 {
        match register {
            Register::AF => self.get_af(),
            Register::BC => self.get_bc(),
            Register::DE => self.get_de(),
            Register::HL => self.get_hl(),
            _ => panic!("Tried to access a non 16-bit register pair: {:?}", register),
        }
    }

    create_pair!(get_af, set_af, a, f);
    create_pair!(get_bc, set_bc, b, c);
    create_pair!(get_de, set_de, d, e);
    create_pair!(get_hl, set_hl, h, l);
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test getters and setters for the word registers. Read above for details on these 16-bit
    // registers backed by two 8-bit registers.
    macro_rules! test_pair {
        ($getname:ident, $setname:ident, $reg1:ident, $reg2:ident) => {
            #[test]
            fn $getname() {
                let mut reg = Registers::new();
                reg.$reg1 = 0xFF;
                reg.$reg2 = 0x11;
                assert_eq!(reg.$getname(), 0xFF11)
            }

            #[test]
            fn $setname() {
                let mut reg = Registers::new();
                reg.$setname(0xFF11);
                assert_eq!(reg.$reg1, 0xFF);
                assert_eq!(reg.$reg2, 0x11);
            }
        };
    }

    test_pair!(get_af, set_af, a, f);
    test_pair!(get_bc, set_bc, b, c);
    test_pair!(get_de, set_de, d, e);
    test_pair!(get_hl, set_hl, h, l);
}
