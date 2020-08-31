pub struct TimerRegisters {
    pub divider: u8,
    pub counter: u8,
    pub modulo: u8,
}

impl TimerRegisters {
    pub fn new() -> Self {
        Self {
            divider: 0,
            counter: 0,
            modulo: 0,
        }
    }

    pub fn rb(&self, address: u16) -> u8 {
        match address {
            0xFF04 => self.divider,
            0xFF05 => self.counter,
            0xFF06 => self.modulo,
            _ => panic!("Tried to read from invalid Timer register: {:x}", address),
        }
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => self.divider = 0,
            0xFF05 => self.counter = value,
            0xFF06 => self.modulo = value,
            _ => panic!("Tried to write to invalid Timer register: {:x}", address),
        }
    }
}
