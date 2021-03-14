use super::MMU;

pub struct APU {
    clock: usize,
}

impl APU {
    pub fn new() -> Self {
        Self { clock: 0 }
    }

    pub fn step(&mut self, mmu: &mut MMU, cycles: u8) {
        // Advance clock by the amount of cycles the CPU ran for.
        self.clock += cycles as usize;

        // When we have passed  4mhz / 44khz worth of cycles, do build samples.
    }

    fn build_samples(&self, mmu: &MMU) {
        // Build one sample for each voice.
    }
}
