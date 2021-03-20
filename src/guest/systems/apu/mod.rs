use std::collections::VecDeque;
mod square;
mod wave;
use super::MMU;
use crate::emulator::{AUDIO_FREQ, CPU_FREQ};
use square::SquareVoice;
use wave::WaveVoice;

// Making this a slight bit lower means we issue samples slightly more often.
// This should result in the audio buffer very slowly falling out of sync as it grows.
// But if we don't do this, there's gaps in audio, even if we queue up a bunch of quiet ahead of time.
const CYCLES_PER_SAMPLE: usize = (CPU_FREQ / AUDIO_FREQ) - 1;

pub struct APU {
    clock: usize,
    square_1: SquareVoice,
    wave: WaveVoice,
    pub output_buffer: VecDeque<[f32; 2]>,
}

impl APU {
    pub fn new() -> Self {
        Self {
            square_1: SquareVoice::new(),
            wave: WaveVoice::new(),
            clock: 0,
            output_buffer: VecDeque::new(),
        }
    }

    pub fn step(&mut self, mmu: &mut MMU, cycles: u8) {
        // TODO: if mmu.apu.enabled is false, don't do anything.

        // Advance clock by the amount of cycles the CPU ran for.
        self.clock += cycles as usize;

        // If 1 audio sample worth of cycles has passed, let's build a sample.
        if self.clock >= CYCLES_PER_SAMPLE {
            // let sample_1 = self.tone.tick()
            let wave_sample = self.wave.build_sample(mmu, cycles);

            // Consume a sample's worth off the clock.
            self.clock -= CYCLES_PER_SAMPLE
        }
    }
}
