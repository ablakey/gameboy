use std::collections::VecDeque;
mod square;
mod wave;
use super::MMU;
use crate::emulator::{APU_DIVISOR, CPU_FREQ};
use square::SquareVoice;
use wave::WaveVoice;

// The number of CPU cycles (4MHz) per frame sequencer frame. Each frame is 64hz.
// Frame sequencer runs at 512hz. There's 1024 CPU cycles per frame. 8 frames per second.
const CYCLES_PER_FRAME: usize = CPU_FREQ / (512 * 8);

pub struct APU {
    clock: usize,
    square_1: SquareVoice,
    wave: WaveVoice,
    frame_sequence: usize,
    pub output_buffer: VecDeque<[f32; 2]>,
}

impl APU {
    pub fn new() -> Self {
        Self {
            square_1: SquareVoice::new(),
            wave: WaveVoice::new(),
            frame_sequence: 0,
            clock: 0,
            output_buffer: VecDeque::new(),
        }
    }

    pub fn step(&mut self, mmu: &mut MMU, cycles: u8) {
        // TODO: if mmu.apu.enabled is false, don't do anything.

        self.clock += cycles as usize;

        // If one "frame" worth of cycles have passed, advance the frame sequencer.
        if self.clock >= CYCLES_PER_FRAME {
            self.clock -= CYCLES_PER_FRAME;

            self.frame_sequence = (self.frame_sequence + 1) % 8;

            // Decrement length counters?
            if [0, 2, 4, 6].contains(&self.frame_sequence) {
                // TODO
            }

            // Decrement sweep?
            if self.frame_sequence == 2 || self.frame_sequence == 6 {
                // TODO
            }

            // Decrement volume envelope?
            if self.frame_sequence == 7 {
                // TODO
            }
        }

        // Run at 1MHz for performance reasons. This means that every tick is 4 cycles.
        // The effect of sound alising is minimal and this can probably be turned further down.
        // If we were to run it too slowly, we would get aliasing, which is when we output one
        // sample that's all one value, when in reality it would have been a mix between multiple
        // values. This affects some voices more than others.
        for _ in 0..(cycles as usize / APU_DIVISOR) {
            let wave_sample = self.wave.tick(mmu);

            // TODO: combine samples
            // TODO: append samples to the output.

            // print!("{}", wave_sample);
            self.output_buffer.push_back([wave_sample, wave_sample]);
        }

        // // If 1 audio sample worth of cycles has passed, let's build a sample.
        // if self.clock >= CYCLES_PER_SAMPLE {
        //     // let sample_1 = self.tone.tick()
        //     let wave_sample = self.wave.build_sample(mmu, cycles);

        //     // Consume a sample's worth off the clock.
        //     self.clock -= CYCLES_PER_SAMPLE
        // }
    }
}

// I need to advance the voices at 1 MHz (so 1/4 of the cycles coming in)
// Generate samples at 1MHz and then downsample them (just pop off an amount and make an average)
//

//  let period = 2 * (2048 - freq_val as i32);
// the period is how many hz it takes until  we advance things (like going to the next wave sample)
// Our implementation will be 1 tick per call (at 1MHz) so it's literall how many times we call tick()
