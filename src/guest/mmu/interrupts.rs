pub struct Interrupts {
    // Both `inte` and `intf` have the same meaning for bits 0-4.  Bits 5-7 are unused.
    // Bit 4: Gamepad high to low
    // Bit 3: Serial I/O transfer complete
    // Bit 2: Timer Overflow
    // Bit 1: LCDC
    // Bit 0: V-Blank
    pub inte: u8, // Address 0xFFFF. Interrupt Enable Switches (is the interrupt enabled?)
    pub intf: u8, // Address 0xFF0F. Interrupt Flags (is the interrupt triggered?)
    pub is_halted: bool,

    // Interrupt Master Enable. Modified via  EI and DI ops, not accessible by address.
    // When a call to disable or enable IME is made, it is done _after_ the _next_ opcode. This
    // means that a call to `disable_ime` sets the `disable_ime_counter` to 2.  On the next opcode,
    // it is reduced to 1.  On the opcode after, the ime is disabled (or enabled) and reset to 0.
    // Two timers exist because it is possible to call both: disable then re-enable before disabled.
    // This has the effect of re-enabling an opcode after.
    ime: bool,
    disable_ime_counter: u8,
    enable_ime_counter: u8,
}

impl Interrupts {
    pub fn new() -> Self {
        Self {
            is_halted: false,
            inte: 0,
            intf: 0,
            ime: true,
            disable_ime_counter: 0,
            enable_ime_counter: 0,
        }
    }

    pub fn disable_ime(&mut self) {
        self.disable_ime_counter = 2;
    }

    pub fn enable_ime(&mut self, delay: u8) {
        self.enable_ime_counter = delay;
    }

    /// Called at the start of every cycle to tick IME timers down and possibly modify the IME.
    pub fn tick_ime_timer(&mut self) {
        self.disable_ime_counter = match self.disable_ime_counter {
            2 => 1,
            1 => {
                self.ime = false;
                0
            }
            _ => 0,
        };

        self.enable_ime_counter = match self.enable_ime_counter {
            2 => 1,
            1 => {
                self.ime = true;
                0
            }
            _ => 0,
        };
    }

    /// Try to handle an interrupt, if any.
    /// This happens on every CPU step, but most of the time returns 0 as there's no interrupt
    /// to handle. Returns an interrupt index if an interrupt is to be handled.
    pub fn try_interrupt(&mut self) -> Option<u8> {
        // If IME is disabled and we're not halted, there isnt any interrupt handling to do.
        if !self.ime && !self.is_halted {
            return None;
        }

        // Get the bitwise intersection of interrupts that are enabled AND have their flag set.
        let active_interrupts = self.inte & self.intf;

        // No interupt flag was set.
        if active_interrupts == 0 {
            return None;
        }

        // Reset halted.  There's more complexity here that we aren't handling right now. See:
        // https://rednex.github.io/rgbds/gbz80.7.html#HALT
        self.is_halted = false;

        if self.intf > 0b11111 {
            panic!(
                "INTF is set to an invalid value. The top 3 bits should always be zero. {:#b}",
                self.intf
            )
        }

        // Isolate which flag to handle by priority (LSB is highest priority).
        // The number of zeroes on the right side equals the index of the highest priority flag.
        let flag_index = active_interrupts.trailing_zeros() as u8;

        // Reset flag.  The flag is inverted to create a mask: everything is reset that isn't set.
        self.intf &= !(1 << flag_index);

        Some(flag_index) // 1,2,3,4,5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_interrupt() {
        let mut interrupts = Interrupts::new();

        // Nothing.
        let result = interrupts.try_interrupt();
        assert_eq!(result, None);

        // Enable some interrupts.
        interrupts.inte = 0b00011100;
        let result = interrupts.try_interrupt();
        assert_eq!(result, None);

        // Set some interrupt flags high.
        interrupts.intf = 0b00010100;
        let result = interrupts.try_interrupt();
        assert_eq!(result, Some(2));
        assert_eq!(interrupts.intf, 0b00010000);

        // Call another interrupt.
        let result = interrupts.try_interrupt();
        assert_eq!(result, Some(4));
        assert_eq!(interrupts.intf, 0b00000000);

        // Set IME to false and try again.
        interrupts.intf = 0b00010100;
        interrupts.ime = false;
        let result = interrupts.try_interrupt();
        assert_eq!(result, None);
        assert_eq!(interrupts.intf, 0b00010100);
    }

    #[test]
    fn test_disable_ime() {
        let mut interrupts = Interrupts::new();

        // Count down.
        interrupts.disable_ime();
        assert_eq!(interrupts.disable_ime_counter, 2);
        interrupts.tick_ime_timer();
        assert_eq!(interrupts.disable_ime_counter, 1);
        interrupts.tick_ime_timer();
        assert_eq!(interrupts.disable_ime_counter, 0);
        interrupts.tick_ime_timer();
        assert_eq!(interrupts.disable_ime_counter, 0);

        // Spam and reset.
        interrupts.disable_ime();
        interrupts.disable_ime();
        interrupts.disable_ime();
        assert_eq!(interrupts.disable_ime_counter, 2);
        interrupts.tick_ime_timer();
        interrupts.disable_ime();
        assert_eq!(interrupts.disable_ime_counter, 2);
    }

    #[test]
    fn test_enable_ime() {
        let mut interrupts = Interrupts::new();

        // Count down.
        interrupts.enable_ime(2);
        assert_eq!(interrupts.enable_ime_counter, 2);
        interrupts.tick_ime_timer();
        assert_eq!(interrupts.enable_ime_counter, 1);
        interrupts.tick_ime_timer();
        assert_eq!(interrupts.enable_ime_counter, 0);
        interrupts.tick_ime_timer();
        assert_eq!(interrupts.enable_ime_counter, 0);

        // Spam and reset.
        interrupts.enable_ime(2);
        interrupts.enable_ime(2);
        interrupts.enable_ime(2);
        assert_eq!(interrupts.enable_ime_counter, 2);
        interrupts.tick_ime_timer();
        interrupts.enable_ime(2);
        assert_eq!(interrupts.enable_ime_counter, 2);

        // Enable 1.
        interrupts.enable_ime(1);
        assert_eq!(interrupts.enable_ime_counter, 1);
        interrupts.tick_ime_timer();
        assert_eq!(interrupts.enable_ime_counter, 0);
    }
}
