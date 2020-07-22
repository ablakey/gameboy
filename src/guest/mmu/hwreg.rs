/// TODO: explain that the public structs are the interface for the hardware to manipulate
/// the registers.  The MMU should only talk to the registers through get/set.
pub struct HardwareRegisters {
    // APU Registers
    nr11: u8, // Sound mode 1 length/wave.
    nr12: u8, // Sound mode 1 envelope.
    nr13: u8, // Sound mode 1 register, frequency Lo
    nr14: u8, // Sound mode 1 register, frequency High
    nr50: u8, // Channel control, on/off, volume.
    nr51: u8, // Selection of Sound output terminal.
    nr52: u8, // Power to sound.

    // PPU Registers
    scy: u8,    // scroll Y background.
    bgp: u8,    // background & window palette details.
    lcdc: u8,   // LCD display flags.
    pub ly: u8, // vertical line data is transferred to. 0-153, 144-153 are during vblank.
    pub lyc: u8,
    pub mode: u8,
}

impl HardwareRegisters {
    pub fn new() -> Self {
        Self {
            bgp: 0,
            lcdc: 0,
            ly: 0,
            lyc: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            nr50: 0,
            nr51: 0,
            nr52: 0,
            scy: 0,
            mode: 0,
        }
    }

    pub fn set(&mut self, address: u16, value: u8) {
        match address {
            0xFF11 => self.nr11 = value,
            0xFF12 => self.nr12 = value,
            0xFF13 => self.nr13 = value,
            0xFF14 => self.nr14 = value,
            0xFF24 => self.nr50 = value,
            0xFF25 => self.nr51 = value,
            0xFF26 => self.nr52 = value,
            0xFF40 => self.lcdc = value,
            0xFF42 => self.scy = value,
            0xFF44 => panic!("Cannot set LY"),
            0xFF47 => self.bgp = value,
            _ => panic!(
                "Tried to set a hardware register with invalid address {:x}",
                address
            ),
        }
    }

    /// Return an 8-bit value when reading from a given address. Some hardware register addresses
    /// are not readable.
    pub fn get(&self, address: u16) -> u8 {
        match address {
            0xFF40 => self.lcdc,
            0xFF41 => {
                (if false { 0x40 } else { 0 }) // TODO: actually handle the registers.
                    | (if false { 0x20 } else { 0 }) // TODO: actually handle the registers.
                    | (if false { 0x10 } else { 0 }) // TODO: actually handle the registers.
                    | (if false { 0x08 } else { 0 }) // TODO: actually handle the registers.
                    | (if self.ly == self.lyc { 0x04 } else { 0 })
                    | self.mode
            }
            0xFF42 => self.scy,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            _ => panic!(
                "Tried to get a hardware register wtih invalid address {:x}",
                address
            ),
        }
    }
}
