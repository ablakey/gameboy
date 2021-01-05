use super::super::MMU;

/// Logical exclusive OR n with register A, result stored in A.
/// Flags: [Z 0 0 0]
pub fn xor(mmu: &mut MMU, n: u8) {
    mmu.a ^= n;
    mmu.set_flag_z(mmu.a == 0);
    mmu.set_flag_n(false);
    mmu.set_flag_h(false);
    mmu.set_flag_c(false);
}

/// Logical OR n with register A, result stored in A.
/// Flags: [Z 0 0 0]
pub fn or(mmu: &mut MMU, n: u8) {
    mmu.a |= n;
    mmu.set_flag_z(mmu.a == 0);
    mmu.set_flag_n(false);
    mmu.set_flag_h(false);
    mmu.set_flag_c(false);
}

/// Logical AND n with register A, result stored in A.
/// Flags [Z 0 1 0]
pub fn and(mmu: &mut MMU, n: u8) {
    mmu.a &= n;
    mmu.set_flag_z(mmu.a == 0);
    mmu.set_flag_n(false);
    mmu.set_flag_h(true);
    mmu.set_flag_c(false);
}

/// Increment register value. Set Z if zero, H if half carry (bit 3), N reset.
/// Not to be used for INC r16 (eg. INC DE) as those do not have flag effects.
/// Flags: [Z 0 H -]
pub fn inc(mmu: &mut MMU, value: u8) -> u8 {
    let new_value = value.wrapping_add(1);

    // Calculate a half-carry by isolating the low nibble, adding one, and seeing if the result
    // is larger than 0xF (fourth bit is high).
    mmu.set_flag_z(new_value == 0);
    mmu.set_flag_n(false);
    mmu.set_flag_h(((0xF & value) + 1) > 0xF);

    new_value
}

/// Decrement value by 1.
/// Flags: [Z 1 H -]
pub fn dec(mmu: &mut MMU, value: u8) -> u8 {
    let new_value = value.wrapping_sub(1);

    mmu.set_flag_z(new_value == 0);
    mmu.set_flag_n(true);

    // There's a half borrow (bit 4) if bits 0-3 have nothing to borrow.
    mmu.set_flag_h((0x0F & value) == 0);

    new_value
}

/// Test if a specific bit of a byte is high or low. If low, set Z (zero flag).
/// Flags: [Z 0 1 -]
pub fn bit(mmu: &mut MMU, bit_index: u8, value: u8) {
    let mask = 0b1 << bit_index;
    let is_unset = value & mask == 0;
    mmu.set_flag_z(is_unset);
    mmu.set_flag_n(false);
    mmu.set_flag_h(true);
}

/// Add value to A.
/// See sub to better understand things about half-carry and half-borrow, etc.
/// Carry is calculated by expanding the upper bounds and seeing if the result sum is > 255.
/// Half-carry is calculated by isolating the lower nibble and seeing if the sum exceeds 15.
/// Flags: [Z 0 H C]
pub fn add(mmu: &mut MMU, value: u8) {
    let (new_a, overflow) = mmu.a.overflowing_add(value);
    mmu.set_flag_z(new_a == 0);
    mmu.set_flag_n(false);
    mmu.set_flag_h((mmu.a & 0xF) + (value & 0xF) > 0xF);
    mmu.set_flag_c(overflow);
    mmu.a = new_a;
}

/// Add 16-bit value to HL.
/// The half-carry is for overflow out of bit 11. That's calculated by isolating bit 11 with a mask
/// then seeing if the sum is greater than 0x7FF (ie. there's a value in any bit above 11).
/// The carry is the same concept but for bit 15. Instead of causing an overflow, we just check to
/// see if there would be one.
/// Flags: [- 0 H C]
pub fn add_hl_16(mmu: &mut MMU, value: u16) {
    let hl = mmu.hl();
    let (new_hl, overflow) = hl.overflowing_add(value);
    mmu.set_flag_n(false);
    mmu.set_flag_h((hl & 0x07FF) + (value & 0x07FF) > 0x07FF);
    mmu.set_flag_c(hl > 0xFFFF - value);
    mmu.set_flag_c(overflow);
    mmu.set_hl(new_hl);
}

/// Subtract value from A.
/// H is set if a half borrow occurs. This is calculated by isolating just the bottom nibble
/// and calculating a full borrow of that. This is done by seeing if the operand is greater than
/// self.a, because that means there would be a wrap around (aka a borrow happens).
/// C is set if there is a full borrow. Same method for detecting: is the operand larger?
/// Flags: [Z 1 H C]
pub fn sub(mmu: &mut MMU, value: u8) {
    let new_a = mmu.a.wrapping_sub(value);
    mmu.set_flag_z(new_a == 0);
    mmu.set_flag_n(true);
    mmu.set_flag_h((mmu.a & 0x0F) < (value & 0x0F));
    mmu.set_flag_c(mmu.a < value);
    mmu.a = new_a;
}

/// Subtract value and the carry bit from A.
pub fn sbc(mmu: &mut MMU, value: u8) {
    sub(mmu, value + mmu.flag_c() as u8);
}

/// Rotate bits left through carry.
/// This means that we shift left, and the MSB becomes the LSB. Except "through carry" means
/// We act as if the carry is part of that ring: MSB becomes carry, old carry becomes LSB.
/// Flags: [Z 0 0 C]
// Note: The mnemonic is weird.  RL is through carry. RLC is not.
pub fn rl(mmu: &mut MMU, value: u8) -> u8 {
    let new_value = value << 1 | mmu.flag_c() as u8;
    mmu.set_flag_z(new_value == 0);
    mmu.set_flag_h(false);
    mmu.set_flag_n(false);
    mmu.set_flag_c((value & 0x80) == 0x80); // If the value's MSB is 1, there's a carry.
    new_value
}

/// Rotate bits left.
/// Flags: [Z 0 0 C]
pub fn rlc(mmu: &mut MMU, value: u8) -> u8 {
    let has_carry = value & 0x80 == 0x80;
    let new_value = value << 1 | has_carry as u8;
    mmu.set_flag_z(new_value == 0);
    mmu.set_flag_h(false);
    mmu.set_flag_n(false);
    mmu.set_flag_c(has_carry); // If the value's MSB is 1, there's a carry.
    new_value
}

/// Rotate bits right through carry.
/// This means that we shift right, and the MSB becomes the LSB. Except "through carry" means
/// We act as if the carry is part of that ring: MSB becomes carry, old carry becomes LSB.
/// Flags: [Z 0 0 C]
// Note: The mnemonic is weird.  RR is through carry. RRC is not.
pub fn rr(mmu: &mut MMU, value: u8) -> u8 {
    let has_carry = value & 0x01 == 0x01;
    let new_value = value >> 1 | value & if mmu.flag_c() { 0x80 } else { 0x00 };
    mmu.set_flag_z(new_value == 0);
    mmu.set_flag_h(false);
    mmu.set_flag_n(false);
    mmu.set_flag_c(has_carry); // If the value's LSB is 1, there's a carry.
    new_value
}

/// Rotate bits right.
/// Flags: [Z 0 0 C]
pub fn rrc(mmu: &mut MMU, value: u8) -> u8 {
    let has_carry = value & 0x01 == 0x01;
    let new_value = value >> 1 | if has_carry { 0x80 } else { 0x00 };
    mmu.set_flag_z(new_value == 0);
    mmu.set_flag_h(false);
    mmu.set_flag_n(false);
    mmu.set_flag_c(has_carry); // If the value's MSB is 1, there's a carry.
    new_value
}

/// Subtract value from A and update registers. Do not change A. This is used as a way to compare
/// values, given the flags change, a program can then look at the flags (usually Z) to see
/// if the result was zero or not.
/// Flags: [Z 1 H C]
pub fn cp(mmu: &mut MMU, value: u8) {
    mmu.set_flag_z(mmu.a.wrapping_sub(value) == 0);
    mmu.set_flag_n(true);
    mmu.set_flag_h((mmu.a & 0x0F) < (value & 0x0F));
    mmu.set_flag_c(mmu.a < value);
}

/// Complement A.
/// Flags: [- 1 1 -]
pub fn cpl(mmu: &mut MMU) {
    mmu.a = !mmu.a;
    mmu.set_flag_n(true);
    mmu.set_flag_h(true);
}

/// Swap upper four and lower four bits.
/// Note that the zero flag is equivalent to if the value is zero. Swapping bits won't change
/// anything if it's zero.
/// Flags: [Z 0 0 0]
pub fn swap(mmu: &mut MMU, value: u8) -> u8 {
    mmu.set_flag_z(value == 0);
    mmu.set_flag_n(false);
    mmu.set_flag_h(false);
    mmu.set_flag_c(false);
    // Swap by shifting MSBs down, LSBs up, and unioning the two results.
    (value >> 4) | (value << 4)
}

/// Reset bit in input value. For example, if value was register A of 0xFF and we reset bit 0, the
/// resulting value would be 0xFE.
/// Flags: [- - - -]
pub fn res(bit: u8, value: u8) -> u8 {
    value & !(1 << bit)
}

/// SEt bit in input value. For example, if value was register A of 0x00 and we set bit 0, the
/// resulting value would be 0x01.
/// Flags: [- - - -]
pub fn set(bit: u8, value: u8) -> u8 {
    value | (1 << bit)
}

/// Shift Left Arithmetic.
/// This means to shift everything left by 1.  The MSB gets set on C (carry) and the LSB is 0.
/// Flags: [Z 0 0 C]
pub fn sla(mmu: &mut MMU, value: u8) -> u8 {
    let new_value = value << 1;
    mmu.set_flag_z(new_value == 0);
    mmu.set_flag_n(false);
    mmu.set_flag_h(false);
    mmu.set_flag_c(value >> 7 == 1);
    new_value
}

/// Shift Right Arithmetic.
/// This means to shift everything right by 1.  The LSB gets set on C (carry) and the MSB is 0.
/// Flags: [Z 0 0 C]
pub fn sra(mmu: &mut MMU, value: u8) -> u8 {
    let msb = value & 0x80;
    let new_value = (value >> 1) | msb; // Populate new MSB with whatever was in MSB.
    mmu.set_flag_z(new_value == 0);
    mmu.set_flag_n(false);
    mmu.set_flag_h(false);
    mmu.set_flag_c(value & 0x01 == 0x01); // LSB was high, there's a carry (borrow).
    new_value
}

/// Shift Right Logic.
/// Flags: [Z 0 0 C]
/// TODO: tests.
pub fn srl(mmu: &mut MMU, value: u8) -> u8 {
    let new_value = value >> 1;
    mmu.set_flag_z(new_value == 0);
    mmu.set_flag_n(false);
    mmu.set_flag_h(false);
    mmu.set_flag_c(value & 0x01 == 0x01); // LSB was high, there's a carry (borrow).
    return new_value;
}

/// Add value plus carry flag to A.
/// Almost the same as alu::add. Duplicated it here to keep both simple for learning purposes.
/// Flags: [Z 0 H C]
pub fn adc(mmu: &mut MMU, value: u8) {
    let value_with_carry = value.wrapping_add(mmu.flag_c() as u8);
    let (new_a, overflow) = mmu.a.overflowing_add(value_with_carry);
    mmu.set_flag_z(new_a == 0);
    mmu.set_flag_n(false);
    mmu.set_flag_h((mmu.a & 0xF) + (value & 0xF) > 0xF);
    mmu.set_flag_c(overflow);
    mmu.a = new_a;
}

/// Decimal Adjust Accumulator
/// I've implemented this fairly verbosely to make it more clear to walk through. This one stuck
/// me for a while. I borrowed inspiration from rboy as well as some Googling that showed C and C#
/// implementations. Make sure you understand Binary-Coded Decimal (BCD).
///
/// Note that I mean "BCD carry" as in a nibble, which can represent 0-15 is representing
/// A BCD value of greater than 9, the value is wrong and needs to be adjusted. This is the core
/// concept of this operation.
///
/// Flags: [Z - 0 C]
/// TODO: tests
pub fn daa(mmu: &mut MMU) {
    let subtract = mmu.flag_n();
    let carry = mmu.flag_c();
    let halfcarry = mmu.flag_h();

    // Last operation was a subtraction.
    // subtract 6 from both nibbles if carry or half-carry 0x60 to the top-half).
    if subtract {
        if carry {
            mmu.a = mmu.a.wrapping_sub(0x60);
        }
        if halfcarry {
            mmu.a = mmu.a.wrapping_sub(0x6);
        }
    // Last operation was an addition.
    // Add 6 to both nibbles if carry or half-carry 0x60 to the top-half).
    } else {
        // If there was an arithmetic carry, or there already is a BCD carry...
        // 0x99 is a 9 in both nibbles. Adjusting BCD positively would make 99 -> 100 or more.
        if carry || mmu.a > 0x99 {
            mmu.a = mmu.a.wrapping_add(0x60);
            mmu.set_flag_c(true);
        }

        // If there was an arithmetic half-carry or if there is already a BCD carry...
        // We mask the smaller nibble and see if it's greater than 9.
        if halfcarry || (mmu.a & 0x0F) > 0x09 {
            mmu.a = mmu.a.wrapping_add(0x6);
        }
    }

    mmu.set_flag_z(mmu.a == 0);
    mmu.set_flag_h(false);
    // mmu.set_flag_c(tru);
}

#[cfg(test)]
mod tests {
    use super::*;

    ///Assert that all flags are certain values.
    /// Z N H C
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
    fn test_inc() {
        let mmu = &mut MMU::new(None, true);
        mmu.a = 0xFF;
        mmu.a = inc(mmu, mmu.a);
        assert_eq!(mmu.a, 0x0);
        assert_flags!(mmu, true, false, true, false);
    }

    #[test]
    fn test_dec() {
        let mmu = &mut MMU::new(None, true);
        mmu.a = 0x10; // There will be a half-borrow.
        mmu.a = dec(mmu, mmu.a);
        assert_eq!(mmu.a, 0x0F);
        assert_flags!(mmu, false, true, true, false);
    }

    #[test]
    fn test_xor() {
        let mmu = &mut MMU::new(None, true);
        xor(mmu, 0x11);
        assert_eq!(mmu.a, 0x11);
        assert_flags!(mmu, false, false, false, false);

        xor(mmu, 0xFF);
        assert_eq!(mmu.a, 0xEE);
        assert_flags!(mmu, false, false, false, false);

        mmu.a = 0x00;
        xor(mmu, 0x00);
        assert_eq!(mmu.a, 0x00);
        assert_flags!(mmu, true, false, false, false);
    }

    #[test]
    fn test_bit() {
        let mmu = &mut MMU::new(None, true);
        mmu.a = 0b00001000;
        bit(mmu, 3, mmu.a);
        assert_flags!(mmu, false, false, true, false);

        mmu.a = 0b00000000;
        bit(mmu, 3, mmu.a);
        assert_flags!(mmu, true, false, true, false);
    }

    #[test]
    fn test_sub() {
        let mmu = &mut MMU::new(None, true);
        mmu.a = 0x10;
        sub(mmu, 0xFF);
        assert_eq!(mmu.a, 0x11);
        assert_flags!(mmu, false, true, true, true);
    }

    #[test]
    fn test_sub_no_borrows() {
        let mmu = &mut MMU::new(None, true);
        mmu.a = 0xFF;
        sub(mmu, 0xFF);
        assert_eq!(mmu.a, 0x00);
        assert_flags!(mmu, true, true, false, false);
    }

    #[test]
    fn test_cp() {
        let mmu = &mut MMU::new(None, true);
        mmu.a = 0x10;
        cp(mmu, 0xFF);
        assert_eq!(mmu.a, 0x10); // Does not get changed.
        assert_flags!(mmu, false, true, true, true);
    }

    #[test]
    fn test_cp_no_borrows() {
        let mmu = &mut MMU::new(None, true);
        mmu.a = 0xFF;
        cp(mmu, 0xFF);
        assert_eq!(mmu.a, 0xFF);
        assert_flags!(mmu, true, true, false, false);
    }

    #[test]
    fn test_rl() {
        let mmu = &mut MMU::new(None, true);
        let result = rl(mmu, 0b10000001);

        // MSB becomes carry (c=true), LSB is 0 (carry was false). Shift left.
        assert_eq!(result, 0b00000010);
        assert_flags!(mmu, false, false, false, true);
    }

    #[test]
    fn test_add() {
        let mmu = &mut MMU::new(None, true);
        mmu.a = 0xFF;
        add(mmu, 0xFF);
        assert_eq!(mmu.a, 0xFE);
        assert_flags!(mmu, false, false, true, true);
    }

    #[test]
    fn test_add_no_carry() {
        let mmu = &mut MMU::new(None, true);
        mmu.a = 0x00;
        add(mmu, 0xE);
        assert_eq!(mmu.a, 0xE);
        assert_flags!(mmu, false, false, false, false);
    }

    #[test]
    fn test_or() {
        let mmu = &mut MMU::new(None, true);
        or(mmu, 0x11);
        assert_eq!(mmu.a, 0x11);
        assert_flags!(mmu, false, false, false, false);

        or(mmu, 0xFF);
        assert_eq!(mmu.a, 0xFF);
        assert_flags!(mmu, false, false, false, false);

        mmu.a = 0x00;
        or(mmu, 0x00);
        assert_eq!(mmu.a, 0x00);
        assert_flags!(mmu, true, false, false, false);
    }

    #[test]
    fn test_and() {
        let mmu = &mut MMU::new(None, true);
        and(mmu, 0x11);
        assert_eq!(mmu.a, 0x00);
        assert_flags!(mmu, true, false, true, false);

        and(mmu, 0xFF);
        assert_eq!(mmu.a, 0x00);
        assert_flags!(mmu, true, false, true, false);

        mmu.a = 0xF0;
        and(mmu, 0xF0);
        assert_eq!(mmu.a, 0xF0);
        assert_flags!(mmu, false, false, true, false);

        mmu.a = 0xF0;
        and(mmu, 0xE0);
        assert_eq!(mmu.a, 0xE0);
        assert_flags!(mmu, false, false, true, false);
    }

    #[test]
    fn test_cpl() {
        let mmu = &mut MMU::new(None, true);
        mmu.a = 0b10101100;
        cpl(mmu);
        assert_eq!(mmu.a, 0b01010011); // The inverse of all bits.
        assert_flags!(mmu, false, true, true, false);
    }

    #[test]
    fn test_swap() {
        let mmu = &mut MMU::new(None, true);
        let result = swap(mmu, 0b11110000);
        assert_eq!(result, 0b00001111);
        assert_flags!(mmu, false, false, false, false);
    }

    #[test]
    fn test_add_hl_16() {
        let mmu = &mut MMU::new(None, true);
        add_hl_16(mmu, 0xFFFF);
        assert_eq!(mmu.hl(), 0xFFFF);
        assert_flags!(mmu, false, false, false, false);

        add_hl_16(mmu, 0xFFFF); // Both overflows.
        assert_eq!(mmu.hl(), 0xFFFE);
        assert_flags!(mmu, false, false, true, true);
    }

    #[test]
    fn test_res() {
        assert_eq!(res(0, 0xFF), 0xFE);
        assert_eq!(res(1, 0xFF), 0xFD);
        assert_eq!(res(7, 0xFF), 0x7F);
    }

    #[test]
    fn test_set() {
        assert_eq!(set(0, 0x00), 0x01);
        assert_eq!(set(1, 0x00), 0x02);
        assert_eq!(set(7, 0xFF), 0xFF);
    }

    #[test]
    fn test_sla() {
        let mmu = &mut MMU::new(None, true);
        assert_eq!(sla(mmu, 0b10000001), 0b00000010);
        assert_flags!(mmu, false, false, false, true);

        assert_eq!(sla(mmu, 0b10000000), 0);
        assert_flags!(mmu, true, false, false, true);
    }

    #[test]
    fn test_rlc() {
        let mmu = &mut MMU::new(None, true);
        let result = rlc(mmu, 0b10000001);

        // MSB becomes carry (c=true), LSB is 0 (carry was false). Shift left.
        assert_eq!(result, 0b00000011);
        assert_flags!(mmu, false, false, false, true);
    }

    #[test]
    fn test_adc() {
        let mmu = &mut MMU::new(None, true);
        mmu.set_flag_c(true);
        mmu.a = 0xFF;
        adc(mmu, 0xFF);
        assert_eq!(mmu.a, 0xFF);
        assert_flags!(mmu, false, false, true, false);
    }
}
