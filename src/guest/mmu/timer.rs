use super::is_bit_set;

/// There are two timers: the Divider Register, and the Timer Counter. The Divider is always running
/// while the Counter can be started and stopped.
/// clock (0xFF07) modes:
/// 00: 4.096 KHz
/// 01: 262.144 Khz
/// 10: 65.536 KHz
/// 11: 16.384 KHz
pub struct TimerRegisters {
    pub divider: u8,
    pub counter: u8,
    pub modulo: u8,
    pub started: bool, // 0xFF07 (bit 2) Start/Stop timer.
    pub clock: u8,     // 0xFF07 (bits 0, 1) Timer clock select (4 clock speed options).
}

impl TimerRegisters {
    pub fn new() -> Self {
        Self {
            divider: 0,
            counter: 0,
            modulo: 0,
            started: false,
            clock: 0,
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
            0xFF07 => {
                self.started = is_bit_set(value, 2);
                self.clock = value & 0b11; // Bottom two bits represent one of 4 clock options.
            }
            _ => panic!(
                "Tried to write {:#x} to invalid Timer register: {:#x}",
                value, address
            ),
        }
    }
}
