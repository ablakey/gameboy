use crate::emulator::APU_DIVISOR;

// See: https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#Square_Wave
const DUTY_CYCLES: [[i32; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
    [1, 0, 0, 0, 0, 0, 0, 1], // 25%
    [1, 0, 0, 0, 0, 1, 1, 1], // 50%
    [0, 1, 1, 1, 1, 1, 1, 1], // 75%
];

pub struct SquareVoice {
    clock: usize,      // Track where we are in playing the current phase of the duty_cycle.
    duty_phase: usize, // Track which of the 8 steps in the current duty cycle we're playing.
}

impl SquareVoice {
    pub fn new() -> Self {
        Self {
            clock: 0,
            duty_phase: 0,
        }
    }

    pub fn tick(&mut self, length: u8, frequency: u16, wave_duty: u8) -> f32 {
        if length == 0 {
            return 0.0;
        }

        // TODO: frequency, if we had freq_sweep, could be different.

        let period = (2048 - frequency) * 4;

        if self.clock >= period as usize {
            self.clock = 0;
            self.duty_phase = (self.duty_phase + 1) % 8;
        };

        // We tick at about 1MHz and need to increment the clock at about 4MHz.
        self.clock += APU_DIVISOR;

        let duty_cycle = DUTY_CYCLES[wave_duty as usize];
        let duty_sample = duty_cycle[self.duty_phase];

        duty_sample as f32 * 2.0 - 1.0
    }
}
