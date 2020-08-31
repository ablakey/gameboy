use super::MMU;

pub struct Gamepad {
    button_state: u8, // P15
    dpad_state: u8,   // P14
}

impl Gamepad {
    pub fn new() -> Self {
        Self {
            button_state: 0xF,
            dpad_state: 0xF,
        }
    }

    /// Assemble a row bitfield (bits 0-3 only) from four boolean states.
    /// Note that this inverts the state, given true == keypressed, but 0 == keypressed in the row.
    fn parse_row(keys: &[bool]) -> u8 {
        let mut row = 0u8;
        for (n, &s) in keys.iter().enumerate() {
            if !s {
                row |= 1 << n
            }
        }
        row
    }

    /// Update the gamepad's state given the provided state of all 8 keys.
    /// The array of booleans represents state in order [Right, Left, Up, Down, A, B, Select, Start]
    /// This function is to be called enough to make the input feel crisp but not on every frame.
    /// 60fps is probably a good and simple target.
    pub fn update_state(&mut self, mmu: &mut MMU, new_state: [bool; 8]) {
        self.button_state = Self::parse_row(&new_state[4..]);
        self.dpad_state = Self::parse_row(&new_state[..4]);

        // TODO: interrupts when a button is pressed. Does it happen here or in `step`?
        // If button state is selected, get state goint from high to low for each button.
        // If any of them are true (button was pressed = high to low) then issue an IRQ.
        // Material nonimplication:  a & !b;
    }

    /// On every frame, read the MMU register value (bits 5 and 6) and set bits 0-3 accordingly.
    pub fn step(&self, mmu: &mut MMU) {
        let read_buttons = mmu.gamepad & 0x20;
        let read_dpad = mmu.gamepad & 0x10;

        // Should never be trying to read both or neither.
        assert_ne!(read_buttons, read_dpad);

        // A `0` in bits 4 or 5 represent "selected".
        mmu.gamepad |= if read_buttons == 0 {
            self.button_state
        } else {
            self.dpad_state
        }
    }
}
