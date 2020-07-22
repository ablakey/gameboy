pub struct PPU {
    modeclock: usize, // Current clock step representing where the PPU is in its processing cycle.
                      // An image buffer.
}
use super::MMU;
use log::info;

impl PPU {
    pub fn new() -> Self {
        Self { modeclock: 0 }
    }

    /// TODO: explain the mode cycle and clocks.
    pub fn step(&mut self, mmu: &mut MMU, cycles: u8) {
        let mode = mmu.hwreg.mode;

        info!("Line:{}, H: {}", mmu.hwreg.ly, mmu.h);

        // Increase the clock by number of cycles being emulated. This will govern what needs
        // to happen next such as changing modes. It is possible that we exceed the number of
        // cycles for the current mode. For that reason, we subtract the expected count from
        // self.modeclock. That allows excessive cycles to be carried over to the next mode.
        self.modeclock += cycles as usize;

        // OAM Read mode.
        // Note that we're not doing OAM read here. We're just simulating the amount of time that
        // the original hardware would take to OAM read. This is necessary to keep all timing
        // in sync. When OAM is needed, it will be read at what's effectively instantaneous speed.
        if mode == 2 && self.modeclock >= 80 {
            self.modeclock -= 80;
            mmu.hwreg.mode = 3;
            return;
        }

        // VRAM read mode. End of mode 3 acts as end of scanline.
        if mode == 3 && self.modeclock >= 172 {
            self.modeclock -= 172;
            mmu.hwreg.mode = 0;
            self.draw_scanline();
            return;
        }

        // HBlank. Upon entering this state, we would have successfully drawn a line and are
        // moving on to the next line or vblank.
        if mode == 0 && self.modeclock >= 204 {
            self.modeclock -= 204;
            mmu.hwreg.ly += 1; // Advance 1 line as we're in hblank.

            // At the end of hblank, if on line 143, we've drawn all 144 lines and need to enter
            // vblank. Otherwise go back to mode 2 and loop again.
            if mmu.hwreg.ly == 143 {
                mmu.hwreg.mode = 1;
            } else {
                mmu.hwreg.mode = 2;
            }
        }

        // VBlank. This runs for 10 lines (4560 cycles) and does increment hwreg.ly. It is valid
        // for hwreg.ly to be a value of 144 to 152, representing when it is in vblank.
        if mode == 1 && self.modeclock >= 456 {
            self.modeclock -= 456;

            if mmu.hwreg.ly == 152 {
                mmu.hwreg.mode = 2;
                mmu.hwreg.ly = 0;
            } else {
                mmu.hwreg.ly += 1;
            }
        }
    }

    fn draw_scanline(&self) {}
}
