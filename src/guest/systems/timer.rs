use super::MMU;

/// TODO: explain.
/// Note that I'm using u16 and not usize or larger so that ifg the divider_counter ever gets
/// out of hand, the application crashes (vis overflow) sooner than later. Easier to catch a bug,
pub struct Timer {
    divider_counter: u16,
}

impl Timer {
    pub fn new() -> Self {
        Self { divider_counter: 0 }
    }

    pub fn step(&mut self, mmu: &mut MMU, cycles: u8) {
        self.divider_counter += cycles as u16;
        // Keep track of ticks, incrementing divider when it hits 256.

        // Consumer 256 ticks for every increment of divider.
        while self.divider_counter >= 0x100 {
            mmu.timer_reg.divider = mmu.timer_reg.divider.wrapping_add(1);
            self.divider_counter -= 0x100;
        }
    }
}
