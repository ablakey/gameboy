use super::is_bit_set;

pub struct PpuRegisters {
    // STAT (0xFF41)
    pub lyc_int_enable: bool,   // 0xFF41 (bit 6) LYC  interrupt enable flag.
    pub mode2_int_enable: bool, // 0xFF41 (bit 5) mode 2 interrupt enable flag.
    pub mode1_int_enable: bool, // 0xFF41 (bit 4) mode 1 interrupt enable flag.
    pub mode0_int_enable: bool, // 0xFF41 (bit 3) mode 0 interrupt enable flag.

    pub scy: u8,                // 0xFF42: scroll Y background.
    pub scx: u8,                // 0xFF43: scroll X background.
    pub line: u8,               // 0xFF44: vertical line data is transferred to. 0-153. (LY)
    pub background_palette: u8, // 0xFF47: background & window palette details.
    pub obj_palette_0: u8,      // 0xFF48: OBP0 palette data (sprites).
    pub obj_palette_1: u8,      // 0xFF49: OBP1 palette data (sprites).
    pub win_x: u8,              // 0xFF4A: Window x position.
    pub win_y: u8,              // 0xFF4B: Window y position.
    pub lyc: u8,                // 0xFF45: LCD Y Compare.
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

    pub clear_screen: bool, // Emulator flag: get PPU to clear the screen and reset mode clock.
}

impl PpuRegisters {
    pub fn new() -> Self {
        Self {
            background_palette: 0,
            bg_tilemap: false,
            lcd_on: false,
            line: 0,
            lyc_int_enable: false,
            mode2_int_enable: false,
            mode1_int_enable: false,
            mode0_int_enable: false,
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
            clear_screen: false,
        }
    }

    /// Return an 8-bit value when reading from a given address. Some hardware register addresses
    /// are not readable.
    pub fn rb(&self, address: u16) -> u8 {
        match address {
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
                (if self.lyc_int_enable { 0x40 } else { 0 })
                    | (if self.mode2_int_enable { 0x20 } else { 0 })
                    | (if self.mode1_int_enable { 0x10 } else { 0 })
                    | (if self.mode0_int_enable { 0x08 } else { 0 })
                    | (if self.line == self.lyc { 0x04 } else { 0 })
                    | self.mode
            }
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.line,
            0xFF45 => self.lyc,
            _ => panic!(
                "Tried to get a PPU register wtih invalid address {:x}",
                address
            ),
        }
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            0xFF40 => {
                let was_lcd_on = self.lcd_on;
                // TODO: I think some other values need to be reset if lcd_on gets toggled.
                self.lcd_on = is_bit_set(value, 7);
                self.window_tilemap = is_bit_set(value, 6);
                self.window_on = is_bit_set(value, 5);
                self.tile_data_table = is_bit_set(value, 4);
                self.bg_tilemap = is_bit_set(value, 3);
                self.sprite_size = is_bit_set(value, 2);
                self.sprite_on = is_bit_set(value, 1);
                self.window_bg_on = is_bit_set(value, 0);

                // LCD was turned off.
                if was_lcd_on && !self.lcd_on {
                    self.line = 0;
                    self.mode = 0;
                    self.clear_screen = true;
                }
            }
            0xFF41 => {
                self.lyc_int_enable = is_bit_set(value, 6);
                self.mode2_int_enable = is_bit_set(value, 5);
                self.mode1_int_enable = is_bit_set(value, 4);
                self.mode0_int_enable = is_bit_set(value, 3);
            }
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => panic!("Cannot set hwreg.line"),
            0xFF45 => self.lyc = value,
            0xFF47 => self.background_palette = value,
            0xFF48 => self.obj_palette_0 = value,
            0xFF49 => self.obj_palette_1 = value,
            0xFF4A => self.win_y = value,
            0xFF4B => self.win_x = value,
            _ => panic!(
                "Tried to set a ppu register with invalid address {:#x}, value: {:#x}",
                address, value
            ),
        }
    }
}
