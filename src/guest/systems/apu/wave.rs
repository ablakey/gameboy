use crate::guest::MMU;

pub struct WaveVoice {}

impl WaveVoice {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build_sample(&mut self, mmu: &MMU, cycles: u8) -> f32 {

    }
}


/**
For wave:
- clock is only used by length timer
- it is used to decrement the timer by 1 tick every 256hz
- So on main tick, every 256hz, decrement length timer by 1.

- the length_timer is actually a value held in the MMU. It can be changed by writing to it.
- so the wave system manipulates it from there.

- when we write to the length timer, we are writing 256 minus the byte written to that register.
  this should be handled in the MMU.


*/
