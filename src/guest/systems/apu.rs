use std::collections::VecDeque;

use super::MMU;
use crate::emulator::{AUDIO_FREQ, CPU_FREQ};

const CYCLES_PER_SAMPLE: usize = (CPU_FREQ / AUDIO_FREQ) + 1; // Round up. (ceil usage in const?)

pub struct APU {
    clock: usize,
    pub output_buffer: VecDeque<[f32; 2]>,
    counter: usize,
}

impl APU {
    pub fn new() -> Self {
        Self {
            clock: 0,
            output_buffer: VecDeque::new(),
            counter: 0,
        }
    }

    pub fn step(&mut self, mmu: &mut MMU, cycles: u8) {
        // TODO: if mmu.apu.enabled is false, don't do anything.

        // Advance clock by the amount of cycles the CPU ran for.
        self.clock += cycles as usize;

        // If 1 audio sample worth of cycles has passed, let's build a sample.
        if self.clock >= CYCLES_PER_SAMPLE {
            self.counter += 1 as usize;
            // TODO: this is a random test sample. Probably makes awful noise.
            // let right = rng.gen::<f64>();

            if self.counter > 110 {
                self.counter = 0;
            } else if self.counter > 55 {
                self.output_buffer.push_back([-0.25, -0.25]);
            } else {
                self.output_buffer.push_back([0.25, 0.25]);
            }

            // Consume a sample's worth off the clock.
            self.clock -= CYCLES_PER_SAMPLE
        }
    }
}
