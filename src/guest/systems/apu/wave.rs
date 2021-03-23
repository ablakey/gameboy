use crate::{emulator::APU_DIVISOR, guest::MMU};

// FF1C (NR32) sets audio volume at 0, 100%, 50%, 25% given the value of bits 6 and 5.
const OUTPUT_VOLUME: [f32; 4] = [0.0, 1.0, 0.5, 0.25];

pub struct WaveVoice {
    clock: usize,        // Track where we are in playing the current wave sample.
    sample_index: usize, // Current sample (0-31) being played.
}

impl WaveVoice {
    pub fn new() -> Self {
        Self {
            clock: 0,
            sample_index: 0,
        }
    }

    pub fn tick(&mut self, mmu: &MMU) -> f32 {
        let period = 2 * (2048 - mmu.apu.wave_frequency);

        // If a period has elapsed, reset the clock and advance which sample we're playing.
        if self.clock >= period as usize {
            self.clock = 0;
            self.sample_index = (self.sample_index + 1) % 32;
        }

        self.clock += APU_DIVISOR;

        let volume = OUTPUT_VOLUME[mmu.apu.wave_output as usize];

        // Divide by 15 to convert 4 bit intensity to a value between 0.0 and 1.0. Then multiply
        // by volume as well as convert to between -1.0 and 1.0.
        let sample = mmu.apu.wave_ram[self.sample_index] as f32 / 15.0;
        (sample * 2.0 - 1.0) * volume
    }

    // TODO: build_sample becomes tick. It doesn't take cycles because the things that need to know
    // cycles will be mutated by the frame_sequencer.
}

// For wave:

// wave plays

// - clock is only used by length timer
// - it is used to decrement the timer by 1 tick every 256hz
// - So on main tick, every 256hz, decrement length timer by 1.

// - the length_timer is actually a value held in the MMU. It can be changed by writing to it.
// - so the wave system manipulates it from there.

// - when we write to the length timer, we are writing 256 minus the byte written to that register.
//   this should be handled in the MMU.
