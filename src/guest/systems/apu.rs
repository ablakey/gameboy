use super::MMU;
use crate::emulator::{AUDIO_FREQ, CPU_FREQ};

const CYCLES_PER_SAMPLE: usize = CPU_FREQ / AUDIO_FREQ;

pub struct APU {
    clock: usize,
    pub output_buffer: [[f32; 2]; 256],
    counter: usize,
}

impl APU {
    pub fn new() -> Self {
        Self {
            clock: 0,
            output_buffer: Vec::new(),
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

            if self.counter > 220 {
                self.counter = 0;
            } else if self.counter > 110 {
                self.output_buffer.push([-0.25, -0.25]);
            } else {
                self.output_buffer.push([0.25, 0.25]);
            }

            // Consume a sample's worth off the clock.
            self.clock -= CYCLES_PER_SAMPLE
        }

        // When we have passed  4mhz / 44khz worth of cycles, do build samples.
    }
}
