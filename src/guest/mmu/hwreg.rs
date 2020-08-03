/// Hardware registers are not always a data store. They are often a hardware interface that's
/// memory addressable. For example, Some are read only or write only.  Some have effects by writing
/// to them (such as turning on or off the LCD display for drawing new pixels).
pub struct HardwareRegisters {
    pub bootrom_enabled: bool,
    inte: u8, // Interrupt Enable Register.
    intf: u8, // Interrupt flag.

    apu: ApuRegisters,
    pub ppu: PpuRegisters,
}

pub struct PpuRegisters {
    pub scy: u8,                // 0xFF42: scroll Y background.
    pub scx: u8,                // 0xFF43: scroll X background.
    pub line: u8,               // 0xFF44: vertical line data is transferred to. 0-153.
    pub background_palette: u8, // 0xFF47: background & window palette details.
    pub obj_palette_0: u8,      // 0xFF48: OBP0 palette data.
    pub obj_palette_1: u8,      // 0xFF49: OBP1 palette data.
    pub win_x: u8,              // 0xFF4A: Window x position.
    pub win_y: u8,              // 0xFF4B: Window y position.
    pub lyc: u8,
    pub mode: u8,

    // LCDC (0xFF40)
    pub lcd_on: bool,          // Bit7: Draw picture?
    pub window_tilemap: bool,  // Bit6: 0: 0x9800-0x9BFF, 1: 0x9C00-0x9FFF
    pub window_on: bool,       // Bit5: "Window" off or on.
    pub tile_data_table: bool, // Bit4: 0: 0x8800-0x97FF 1: 0x8000-0x8FFF <- 1 is same area as OBJ
    pub bg_tilemap: bool,      // Bit3: 0: 0x9800-0x9BFF, 1: 0x9C00-0x9FFF
    pub sprite_size: bool,     // Bit2: 0: 8x8, 1: 8x16
    pub sprite_on: bool,       // Bit1: Draw sprites?
    pub window_bg_on: bool,    // Bit0: Draw Window and Background?
}

pub struct ApuRegisters {
    pub nr10: u8, // 0xFF10: Sound more 1 sweep.
    pub nr11: u8, // 0xFF11: Sound mode 1 length/wave.
    pub nr12: u8, // 0xFF12: Sound mode 1 envelope.
    pub nr13: u8, // 0xFF13: Sound mode 1 register, frequency Low.
    pub nr14: u8, // 0xFF14: Sound mode 1 register, frequency High.
    pub nr22: u8, // 0xFF17: Sound mode 2 register, envelope.
    pub nr23: u8, // 0xFF18: Sound mode 2 register, frequency Low.
    pub nr24: u8, // 0xFF19: Sound mode 2 register, frequency High.
    pub nr30: u8, // 0xFF1A: Sound mode 3 register, on/off.
    pub nr42: u8, // 0xFF21: Sound mode 4 register, envelope.
    pub nr44: u8, // 0xFF23: SOund mode 4 register, counter/consecutive.
    pub nr50: u8, // 0xFF24: Channel control, on/off, volume.
    pub nr51: u8, // 0xFF25: Selection of Sound output terminal.
    pub nr52: u8, // 0xFF26: Power to sound.
}

impl HardwareRegisters {
    pub fn new() -> Self {
        Self {
            bootrom_enabled: true, // Starts as enabled.
            inte: 0,
            intf: 0,
            apu: ApuRegisters {
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
            },
            ppu: PpuRegisters {
                background_palette: 0,
                bg_tilemap: false,
                lcd_on: false,
                line: 0,
                lyc: 0,
                mode: 0,
                obj_palette_0: 0,
                obj_palette_1: 0,
                scx: 0,
                scy: 0,
                sprite_on: false,
                sprite_size: false,
                tile_data_table: false,
                win_x: 0,
                win_y: 0,
                window_bg_on: false,
                window_on: false,
                window_tilemap: false,
            },
        }
    }

    pub fn set(&mut self, address: u16, value: u8) {
        match address {
            0xFF00 => (), // TODO: gamepad.
            0xFF01 => (), // TODO: serial write.
            0xFF02 => (), // TODO: serial control.
            0xFF06 => (), // TODO: TMA timer.
            0xFF10 => self.apu.nr10 = value,
            0xFF11 => self.apu.nr11 = value,
            0xFF12 => self.apu.nr12 = value,
            0xFF13 => self.apu.nr13 = value,
            0xFF14 => self.apu.nr14 = value,
            0xFF17 => self.apu.nr22 = value,
            0xFF19 => self.apu.nr24 = value,
            0xFF1A => self.apu.nr30 = value,
            0xFF21 => self.apu.nr42 = value,
            0xFF23 => self.apu.nr44 = value,
            0xFF24 => self.apu.nr50 = value,
            0xFF25 => self.apu.nr51 = value,
            0xFF26 => self.apu.nr52 = value,
            0xFF40 => {
                // TODO: I think some other values need to be reset if lcd_on gets toggled.
                self.ppu.lcd_on = is_bit_set(value, 7);
                self.ppu.window_tilemap = is_bit_set(value, 6);
                self.ppu.window_on = is_bit_set(value, 5);
                self.ppu.tile_data_table = is_bit_set(value, 4);
                self.ppu.bg_tilemap = is_bit_set(value, 3);
                self.ppu.sprite_size = is_bit_set(value, 2);
                self.ppu.sprite_on = is_bit_set(value, 1);
                self.ppu.window_bg_on = is_bit_set(value, 0);
            }
            0xFF41 => (), // TODO: handle setting STAT
            0xFF42 => self.ppu.scy = value,
            0xFF43 => self.ppu.scx = value,
            0xFF44 => panic!("Cannot set hwreg.line"),
            0xFF47 => self.ppu.background_palette = value,
            0xFF48 => self.ppu.obj_palette_0 = value,
            0xFF49 => self.ppu.obj_palette_1 = value,
            0xFF4A => self.ppu.win_y = value,
            0xFF4B => self.ppu.win_x = value,
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
                (if self.ppu.lcd_on { 0x80 } else { 0 })
                    | (if self.ppu.window_tilemap { 0x40 } else { 0 })
                    | (if self.ppu.window_on { 0x20 } else { 0 })
                    | (if self.ppu.tile_data_table { 0x10 } else { 0 })
                    | (if self.ppu.bg_tilemap { 0x08 } else { 0 })
                    | (if self.ppu.sprite_size { 0x04 } else { 0 })
                    | (if self.ppu.sprite_on { 0x02 } else { 0 })
                    | (if self.ppu.window_bg_on { 0x01 } else { 0 })
            }
            0xFF41 => {
                (if false { 0x40 } else { 0 }) // TODO: actually add the registers.
                    | (if false { 0x20 } else { 0 }) // TODO: actually add the registers.
                    | (if false { 0x10 } else { 0 }) // TODO: actually add the registers.
                    | (if false { 0x08 } else { 0 }) // TODO: actually add the registers.
                    | (if self.ppu.line == self.ppu.lyc { 0x04 } else { 0 })
                    | self.ppu.mode
            }
            0xFF42 => self.ppu.scy,
            0xFF43 => self.ppu.scx,
            0xFF44 => self.ppu.line,
            0xFF45 => self.ppu.lyc,
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
