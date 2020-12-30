use crate::emulator::{CPU_FREQ, DIVIDER_FREQ};

use super::MMU;

// How many ticks for each increment of the divider counter.
const DIVIDER_TICKSIZE: usize = CPU_FREQ / DIVIDER_FREQ;

/// The timer implementation emulates a hardware timer by keeping local state of the clock cycle.
/// The counters keep track of how much "time" has accumulated each step of the emulator, and are
/// exhausted by the two timers (divider and counter).
pub struct Timer {
    divider_lapsed: u16,
    counter_lapsed: u16,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            divider_lapsed: 0,
            counter_lapsed: 0,
        }
    }

    pub fn step(&mut self, mmu: &mut MMU, cycles: u8) {
        // Divider.
        self.divider_lapsed += cycles as u16;
        while self.divider_lapsed >= DIVIDER_TICKSIZE as u16 {
            mmu.timer.divider = mmu.timer.divider.wrapping_add(1);
            self.divider_lapsed -= DIVIDER_TICKSIZE as u16;
        }

        // Counter.
        if mmu.timer.started {
            // The timer frequency is actually a function of the CPU (a not implemented CGB mode
            // would double the CPU and therefore all the timer modes would run 2x as well)
            let timer_ticksize = match mmu.timer.clock {
                0 => CPU_FREQ / 1024, // 00: 4.096 KHz
                1 => CPU_FREQ / 16,   // 01: 262.144 Khz
                2 => CPU_FREQ / 64,   // 10: 65.536 KHz
                3 => CPU_FREQ / 256,  // 11: 16.384 KHz
                _ => panic!("TODO"),
            } as u16;

            self.counter_lapsed += cycles as u16;
            while self.counter_lapsed >= timer_ticksize {
                mmu.timer.counter = mmu.timer.counter.wrapping_add(1);
                self.counter_lapsed -= timer_ticksize;

                // Timer has overflowed.
                if mmu.timer.counter == 0 {
                    mmu.timer.counter = mmu.timer.modulo;
                }
            }
        }
    }
}
