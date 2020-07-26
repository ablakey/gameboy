use super::MMU;

/// Logical exclusive OR n with register A, result in A.
pub fn alu_xor(m: &mut MMU, n: u8) {
    m.a ^= n;
    m.set_flag_z(m.a == 0);
    m.set_flag_h(false);
    m.set_flag_n(false);
    m.set_flag_c(false);
}

/// Increment register value. Set Z if zero, H if half carry (bit 3), N reset.
/// Not to be used for INC r16 (eg. INC DE) as those do not have flag effects.
/// Flags: [Z 0 H -]
pub fn alu_inc(m: &mut MMU, value: u8) -> u8 {
    let new_value = value.wrapping_add(1);

    // Calculate a half-carry by isolating the low nibble, adding one, and seeing if the result
    // is larger than 0xF (fourth bit is high).
    m.set_flag_z(new_value == 0);
    m.set_flag_n(false);
    m.set_flag_h(((0xF & value) + 1) > 0xF);

    new_value
}

/// Decrement value by 1.
/// Flags: [Z 1 H -]
pub fn alu_dec(m: &mut MMU, value: u8) -> u8 {
    let new_value = value.wrapping_sub(1);

    m.set_flag_z(new_value == 0);
    m.set_flag_n(true);

    // There's a half borrow (bit 4) if bits 0-3 have nothing to borrow.
    m.set_flag_h((0x0F & value) == 0);

    new_value
}

/// Test a specific bit of a given byte.
/// If the provided bit
pub fn alu_bit(m: &mut MMU, bit_index: u8, value: u8) {
    let mask = 0b1 << bit_index;
    let is_unset = value & mask == 0;
    m.set_flag_z(is_unset);
    m.set_flag_n(false);
    m.set_flag_h(true);
}

/// Subtract value from A.
/// H is set if a half borrow occurs. This is calculated by isolating just the bottom nibble
/// and calculating a full borrow of that. This is done by seeing if the operand is greater than
/// self.a, because that means there would be a wrap around (aka a borrow happens).
/// C is set if there is a full borrow. Same method for detecting: is the operand larger?
/// Flags: [Z 1 H C]
pub fn alu_sub(m: &mut MMU, value: u8) {
    let new_a = m.a.wrapping_sub(value);
    m.set_flag_z(new_a == 0);
    m.set_flag_n(true);
    m.set_flag_h((m.a & 0x0F) < (value & 0x0F));
    m.set_flag_c(m.a < value);
    m.a = new_a;
}

/// Subtract value and the carry bit from A.
pub fn alu_sbc(m: &mut MMU, value: u8) {
    alu_sub(m, value + m.flag_c() as u8);
}

/// Rotate bits left through carry.
/// This means that we shift left, and the MSB becomes the LSB. Except "through carry" means
/// We act as if the carry is part of that ring: MSB becomes carry, old carry becomes LSB.
pub fn alu_rl(m: &mut MMU, value: u8) -> u8 {
    let new_value = value << 1 | m.flag_c() as u8;
    m.set_flag_z(new_value == 0);
    m.set_flag_h(false);
    m.set_flag_n(false);
    m.set_flag_c((value & 0x80) == 0x80); // If the value's MSB is 1, there's a carry.
    new_value
}

/// Subtract value from A and update registers. Do not change A. This is used as a way to compare
/// values, given the flags change, a program can then look at the flags (usually Z) to see
/// if the result was zero or not.
/// Flags: [Z 1 H C]
pub fn alu_cp(m: &mut MMU, value: u8) {
    m.set_flag_z(m.a.wrapping_sub(value) == 0);
    m.set_flag_n(true);
    m.set_flag_h((m.a & 0x0F) < (value & 0x0F));
    m.set_flag_c(m.a < value);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Assert that all flags are certain values.
    /// We use a macro instead of a function so that test failures provide more useful lines.
    /// A normal function will just point to a line up here instead of the offending test. We could
    /// enable a full stack trace but that gets really irritating to wade through while debugging.
    macro_rules! assert_flags {
        ($reg:ident, $z:expr, $n:expr, $h:expr, $c:expr) => {
            assert_eq!($reg.flag_z(), $z, "Flag Z");
            assert_eq!($reg.flag_n(), $n, "Flag N");
            assert_eq!($reg.flag_h(), $h, "Flag H");
            assert_eq!($reg.flag_c(), $c, "Flag C");
        };
    }

    #[test]
    fn test_alu_inc() {
        let mmu = &mut MMU::new(None);
        mmu.a = 0xFF;
        mmu.a = alu_inc(mmu, mmu.a);
        assert_eq!(mmu.a, 0x0);
        assert_flags!(mmu, true, false, true, false);
    }

    #[test]
    fn test_alu_dec() {
        let mmu = &mut MMU::new(None);
        mmu.a = 0x10; // There will be a half-borrow.
        mmu.a = alu_dec(mmu, mmu.a);
        assert_eq!(mmu.a, 0x0F);
        assert_flags!(mmu, false, true, true, false);
    }

    #[test]
    fn test_alu_xor() {
        let mmu = &mut MMU::new(None);
        alu_xor(mmu, 0x11);
        assert_eq!(mmu.a, 0x11);
        assert_eq!(mmu.flag_z(), false);

        alu_xor(mmu, 0xFF);
        assert_eq!(mmu.a, 0xEE);
        assert_eq!(mmu.flag_z(), false);

        mmu.a = 0x00;
        alu_xor(mmu, 0x00);
        assert_eq!(mmu.a, 0x00);
        assert_eq!(mmu.flag_z(), true);
    }

    #[test]
    fn test_alu_bit() {
        let mmu = &mut MMU::new(None);
        mmu.a = 0b00001000;
        alu_bit(mmu, 3, mmu.a);
        assert_flags!(mmu, false, false, true, false);

        mmu.a = 0b00000000;
        alu_bit(mmu, 3, mmu.a);
        assert_flags!(mmu, true, false, true, false);
    }

    #[test]
    fn test_alu_sub() {
        let mmu = &mut MMU::new(None);
        mmu.a = 0x10;
        alu_sub(mmu, 0xFF);
        assert_eq!(mmu.a, 0x11);
        assert_flags!(mmu, false, true, true, true);
    }

    #[test]
    fn test_alu_sub_no_borrows() {
        let mmu = &mut MMU::new(None);
        mmu.a = 0xFF;
        alu_sub(mmu, 0xFF);
        assert_eq!(mmu.a, 0x00);
        assert_flags!(mmu, true, true, false, false);
    }

    #[test]
    fn test_alu_cp() {
        let mmu = &mut MMU::new(None);
        mmu.a = 0x10;
        alu_cp(mmu, 0xFF);
        assert_eq!(mmu.a, 0x10); // Does not get changed.
        assert_flags!(mmu, false, true, true, true);
    }

    #[test]
    fn test_alu_cp_no_borrows() {
        let mmu = &mut MMU::new(None);
        mmu.a = 0xFF;
        alu_cp(mmu, 0xFF);
        assert_eq!(mmu.a, 0xFF);
        assert_flags!(mmu, true, true, false, false);
    }

    #[test]
    fn test_alu_rl() {
        let mmu = &mut MMU::new(None);
        let result = alu_rl(mmu, 0b10000001);

        // MSB becomes carry (c=true), LSB is 0 (carry was false). Shift left.
        assert_eq!(result, 0b00000010);
        assert_flags!(mmu, false, false, false, true);
    }
}
