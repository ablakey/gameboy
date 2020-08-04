pub struct Interrupts {
    pub inte: u8,
    pub intf: u8,
}

impl Interrupts {
    pub fn new() -> Self {
        Self { inte: 0, intf: 0 }
    }
}
