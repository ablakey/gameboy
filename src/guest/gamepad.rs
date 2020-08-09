pub struct Gamepad {
    row_0: u8, // Only bits 0-3 are used.
    row_1: u8, // Only bits 0-3 are used.
}

impl Gamepad {
    pub fn new() -> Self {
        Self {
            row_0: 0x0F,
            row_1: 0x0F,
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
    /// The array of booleans represents state in order:
    /// [Right, Left, Up, Down, A, B, Select, Start]
    pub fn update_state(&mut self, new_state: [bool; 8]) {
        self.row_0 = Self::parse_row(&new_state[..4]);
        self.row_1 = Self::parse_row(&new_state[4..]);
    }
}
