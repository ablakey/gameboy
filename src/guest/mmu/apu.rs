use super::is_bit_set;

pub struct ApuRegisters {
    wram: [u8; 0x10], // 32 4-bit wave pattern samples (16 bytes).
    // Square (with sweep)
    pub s1_sweep_time: u8,
    pub s1_sweep_increase: bool,
    pub s1_sweep_shift: u8,
    nr11: u8, // 0xFF11: Sound mode 1 length/wave.
    nr12: u8, // 0xFF12: Sound mode 1 envelope.
    nr13: u8, // 0xFF13: Sound mode 1 register, frequency Low.
    nr14: u8, // 0xFF14: Sound mode 1 register, frequency High.
    // Square
    nr21: u8, // 0xFF16: Sound mode 2 register, length, wave pattern duty.
    nr22: u8, // 0xFF17: Sound mode 2 register, envelope.
    nr23: u8, // 0xFF18: Sound mode 2 register, frequency Low.
    nr24: u8, // 0xFF19: Sound mode 2 register, frequency High.
    // Wave
    nr30: u8, // 0xFF1A: Sound mode 3 register, on/off.
    nr31: u8, // 0xFF1B: Sound mode 3 register, length.
    nr32: u8, // 0xFF1C: Sound mode 3 register, select output level.
    nr33: u8, // 0xFF1D: Sound mode 3 register, frequency's lower data.
    nr34: u8, // 0xFF1E: Sound mode 3 register, frequency's upper data.
    // Noise
    nr41: u8, // 0xFF20: Sound mode 4 register, length.
    nr42: u8, // 0xFF21: Sound mode 4 register, envelope.
    nr43: u8, // 0xFF22: Sound mode 4 register, polynomial counter.
    nr44: u8, // 0xFF23: Sound mode 4 register, counter/consecutive.
    nr50: u8, // 0xFF24: Channel control, on/off, volume.
    nr51: u8, // 0xFF25: Selection of Sound output terminal.
    nr52: u8, // 0xFF26: Power to sound.
}

impl ApuRegisters {
    pub fn new() -> Self {
        Self {
            s1_sweep_time: 0,
            s1_sweep_increase: false,
            s1_sweep_shift: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
            nr30: 0,
            nr31: 0,
            nr32: 0,
            nr33: 0,
            nr34: 0,
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
            nr50: 0,
            nr51: 0,
            nr52: 0,
            wram: [0; 0x10],
        }
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            0xFF10 => {
                self.s1_sweep_time = (value >> 4) & 0x7;
                self.s1_sweep_increase = is_bit_set(value, 3)
            }
            0xFF11 => self.nr11 = value,
            0xFF12 => self.nr12 = value,
            0xFF13 => self.nr13 = value,
            0xFF14 => self.nr14 = value,
            0xFF16 => self.nr21 = value,
            0xFF17 => self.nr22 = value,
            0xFF18 => self.nr23 = value,
            0xFF19 => self.nr24 = value,
            0xFF1A => self.nr30 = value,
            0xFF1B => self.nr31 = value,
            0xFF1C => self.nr32 = value,
            0xFF1D => self.nr33 = value,
            0xFF1E => self.nr34 = value,
            0xFF20 => self.nr41 = value,
            0xFF21 => self.nr42 = value,
            0xFF22 => self.nr43 = value,
            0xFF23 => self.nr44 = value,
            0xFF24 => self.nr50 = value,
            0xFF25 => self.nr51 = value,
            0xFF26 => self.nr52 = value,
            0xFF30..=0xFF3F => self.wram[(address - 0xFF30) as usize] = value,
            _ => panic!(
                "Tried to write to an APU register that was not implemented: {:x}",
                address
            ),
        }
    }

    pub fn rb(&self, address: u16) -> u8 {
        0
        // TODO: Implement.
    }

    // pub fn rb(&self, address: u16) -> u8 {
    //     match address {
    //         0xFF14 => self.nr14, // TODO: not correct. Only bit 6 can be read?
    //         0xFF19 => self.nr24,
    //         0xFF1E => self.nr34,
    //         0xFF23 => self.nr44,
    //         _ => panic!(
    //             "Tried to get a hardware register wtih invalid address {:x}",
    //             address
    //         ),
    //     }
    // }
}
