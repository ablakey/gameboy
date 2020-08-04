pub struct ApuRegisters {
    pub nr10: u8, // 0xFF10: Sound more 1 sweep.
    pub nr11: u8, // 0xFF11: Sound mode 1 length/wave.
    pub nr12: u8, // 0xFF12: Sound mode 1 envelope.
    pub nr13: u8, // 0xFF13: Sound mode 1 register, frequency Low.
    pub nr14: u8, // 0xFF14: Sound mode 1 register, frequency High.
    pub nr22: u8, // 0xFF17: Sound mode 2 register, envelope.
    pub nr23: u8, // 0xFF18: Sound mode 2 register, frequency Low.
    pub nr24: u8, // 0xFF19: Sound mode 2 register, frequency High.
    pub nr30: u8, // 0xFF1A: Sound mode 3 register, on/off.
    pub nr42: u8, // 0xFF21: Sound mode 4 register, envelope.
    pub nr44: u8, // 0xFF23: SOund mode 4 register, counter/consecutive.
    pub nr50: u8, // 0xFF24: Channel control, on/off, volume.
    pub nr51: u8, // 0xFF25: Selection of Sound output terminal.
    pub nr52: u8, // 0xFF26: Power to sound.
}

impl ApuRegisters {
    pub fn new() -> Self {
        Self {
            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
            nr30: 0,
            nr42: 0,
            nr44: 0,
            nr50: 0,
            nr51: 0,
            nr52: 0,
        }
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            0xFF10 => self.nr10 = value,
            0xFF11 => self.nr11 = value,
            0xFF12 => self.nr12 = value,
            0xFF13 => self.nr13 = value,
            0xFF14 => self.nr14 = value,
            0xFF17 => self.nr22 = value,
            0xFF19 => self.nr24 = value,
            0xFF1A => self.nr30 = value,
            0xFF21 => self.nr42 = value,
            0xFF23 => self.nr44 = value,
            0xFF24 => self.nr50 = value,
            0xFF25 => self.nr51 = value,
            0xFF26 => self.nr52 = value,
            _ => panic!(
                "Tried to write to an APU register that was not implemented: {:x}",
                address
            ),
        }
    }

    pub fn rb(&self, address: u16) -> u8 {
        match address {
            _ => panic!(
                "Tried to get a hardware register wtih invalid address {:x}",
                address
            ),
        }
    }
}
