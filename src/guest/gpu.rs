pub struct GPU {}
use super::MMU;

impl GPU {
    pub fn new() -> Self {
        Self {}
    }

    /// Do a number of steps until GPU has emulated `cycles` worth of work.
    /// `cycles` is how much work the CPU has just done, so the GPU has to immediately catch up.
    pub fn do_steps(&self, cycles: u8, mmu: &mut MMU) {}

    /// Advance one step.
    /// A GPU spends:
    /// 80 clocks in mode 2 (accessing OAM)
    /// 172 clocks in mode 3 (accessing VRAM)
    /// 204 clocks in mode 0
    /// 456 clocks for each single line
    /// 4560 clocks for vblank (10 lines worth)
    /// These are all in
    pub fn step(&self, cycles: u8, mmu: &mut MMU) {}
}
