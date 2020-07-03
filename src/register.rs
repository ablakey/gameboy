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

    /// Get `16-bit register pair AF.
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_af() {
        let mut reg = Register::new();
        reg.a = 0xFF;
        reg.f = 0x11;
        assert_eq!(reg.af(), 0xFF11)
    }
}
