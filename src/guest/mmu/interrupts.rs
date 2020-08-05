pub struct Interrupts {
    pub inte: u8,
    pub intf: u8,
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
            ime: false,
            disable_ime_counter: 0,
            enable_ime_counter: 0,
        }
    }

    pub fn disable_ime(&mut self) {}

    pub fn enable_ime(&mut self) {}

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
                self.ime = false;
                0
            }
            _ => 0,
        };
    }
}
