/// TODO: explain that the public structs are the interface for the hardware to manipulate
/// the registers.  The MMU should only talk to the registers through get/set.
pub struct HardwareRegisters {
    // General Hardware Registers
    pub bootrom_enabled: bool,

    // Interrupt Registers
    inte: u8, // Interrupt Enable Register.
    intf: u8, // Interrupt flag.

    // APU Registers
    nr10: u8, // Sound more 1 sweep.
    nr11: u8, // Sound mode 1 length/wave.
    nr12: u8, // Sound mode 1 envelope.
    nr13: u8, // Sound mode 1 register, frequency Low.
    nr14: u8, // Sound mode 1 register, frequency High.
    nr22: u8, // Sound mode 2 register, envelope.
    nr23: u8, // Sound mode 2 register, frequency Low.
    nr24: u8, // Sound mode 2 register, frequency High.
    nr30: u8, // Sound mode 3 register, on/off.
    nr42: u8, // Sound mode 4 register, envelope.
    nr44: u8, // SOund mode 4 register, counter/consecutive.
    nr50: u8, // Channel control, on/off, volume.
    nr51: u8, // Selection of Sound output terminal.
    nr52: u8, // Power to sound.

    // PPU Registers
    win_x: u8,                  // Window x position.
    win_y: u8,                  // Window y position.
    pub scx: u8,                // scroll X background.
    pub scy: u8,                // scroll Y background.
    pub background_palette: u8, // background & window palette details.
    obj_palette_0: u8,          // OBP0 palette data.
    obj_palette_1: u8,          // OBP1 palette data.
    pub line: u8,               // vertical line data is transferred to. 0-153.
    pub lyc: u8,
    pub mode: u8,

    // LCDC (0xFF40)
    // Note that tile map 0x8800-0x97FF are unsigned, 0x9C00-0x9FFF are signed.
    pub lcd_on: bool,          // Draw picture?
    window_tilemap: bool,      // 0: 0x9800-0x9BFF, 1: 0x9C00-0x9FFF
    window_on: bool,           // "Window" off or on.
    pub tile_data_table: bool, // 0: 0x8800-0x97FF 1: 0x8000-0x8FFF <- 1 is same area as OBJ (Sprites?)
    pub bg_tilemap: bool,      // 0: 0x9800-0x9BFF, 1: 0x9C00-0x9FFF
    sprite_size: bool,         // 0: 8x8, 1: 8x16
    sprite_on: bool,           // Draw sprites?
    window_bg_on: bool,        // Draw Window and Background?
}

impl HardwareRegisters {
    pub fn new() -> Self {
        Self {
            bootrom_enabled: true, // Starts as enabled.
            background_palette: 0,
            obj_palette_0: 0,
            obj_palette_1: 0,
            inte: 0,
            intf: 0,
            line: 0,
            lyc: 0,
            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
            nr30: 0,
            nr42: 0,
            nr44: 0,
            nr50: 0,
            nr51: 0,
            nr52: 0,
            scx: 0,
            scy: 0,
            win_y: 0,
            win_x: 0,
            mode: 0,
            lcd_on: false,
            window_tilemap: false,
            window_on: false,
            tile_data_table: false,
            bg_tilemap: false,
            sprite_size: false,
            sprite_on: false,
            window_bg_on: false,
        }
    }

    pub fn set(&mut self, address: u16, value: u8) {
        match address {
            0xFF00 => (), // TODO: gamepad.
            0xFF01 => (), // TODO: serial write.
            0xFF02 => (), // TODO: serial control.
            0xFF06 => (), // TODO: TMA timer.
            0xFF10 => self.nr10 = value,
            0xFF11 => self.nr11 = value,
            0xFF12 => self.nr12 = value,
            0xFF13 => self.nr13 = value,
            0xFF14 => self.nr14 = value,
            0xFF17 => self.nr22 = value,
            0xFF19 => self.nr24 = value,
            0xFF1A => self.nr30 = value,
            0xFF21 => self.nr42 = value,
            0xFF23 => self.nr44 = value,
            0xFF24 => self.nr50 = value,
            0xFF25 => self.nr51 = value,
            0xFF26 => self.nr52 = value,
            0xFF40 => {
                // TODO: I think some other values need to be reset if lcd_on gets toggled.
                self.lcd_on = is_bit_set(value, 7);
                self.window_tilemap = is_bit_set(value, 6);
                self.window_on = is_bit_set(value, 5);
                self.tile_data_table = is_bit_set(value, 4);
                self.bg_tilemap = is_bit_set(value, 3);
                self.sprite_size = is_bit_set(value, 2);
                self.sprite_on = is_bit_set(value, 1);
                self.window_bg_on = is_bit_set(value, 0);
            }
            0xFF41 => (), // TODO: handle setting STAT
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => panic!("Cannot set hwreg.line"),
            0xFF47 => self.background_palette = value,
            0xFF48 => self.obj_palette_0 = value,
            0xFF49 => self.obj_palette_1 = value,
            0xFF4A => self.win_y = value,
            0xFF4B => self.win_x = value,
            0xFF50 => self.bootrom_enabled = false,
            0xFF0F => self.intf = value,
            0xFF7F => (), // tetris.gb off-by-one error.
            0xFFFF => self.inte = value,
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
            0xFF00 => 0xFF, // TODO: gamepad read.
            0xFF01 => 0xFF, // TODO: serial read.
            0xFF02 => 0xFF, // TODO: serial control.
            0xFF40 => {
                (if self.lcd_on { 0x80 } else { 0 })
                    | (if self.window_tilemap { 0x40 } else { 0 })
                    | (if self.window_on { 0x20 } else { 0 })
                    | (if self.tile_data_table { 0x10 } else { 0 })
                    | (if self.bg_tilemap { 0x08 } else { 0 })
                    | (if self.sprite_size { 0x04 } else { 0 })
                    | (if self.sprite_on { 0x02 } else { 0 })
                    | (if self.window_bg_on { 0x01 } else { 0 })
            }
            0xFF41 => {
                (if false { 0x40 } else { 0 }) // TODO: actually add the registers.
                    | (if false { 0x20 } else { 0 }) // TODO: actually add the registers.
                    | (if false { 0x10 } else { 0 }) // TODO: actually add the registers.
                    | (if false { 0x08 } else { 0 }) // TODO: actually add the registers.
                    | (if self.line == self.lyc { 0x04 } else { 0 })
                    | self.mode
            }
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.line,
            0xFF45 => self.lyc,
            0xFFFF => self.inte,
            _ => panic!(
                "Tried to get a hardware register wtih invalid address {:x}",
                address
            ),
        }
    }
}

fn is_bit_set(value: u8, position: u8) -> bool {
    (value & (1 << position)) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_bit_set() {
        assert!(is_bit_set(0b10000000, 7));
        assert!(is_bit_set(0b11111111, 0));
        assert!(!is_bit_set(0b11111110, 0));
        assert!(!is_bit_set(0b10000000, 6));
    }
}
