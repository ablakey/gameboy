use super::super::mmu::is_bit_set;
use super::MMU;

pub struct PPU {
    modeclock: usize, // Current clock step representing where the PPU is in its processing cycle.
    pub bg_color_zero: [bool; 160], // tracks which pixels in a row have background = 0.
    pub image_buffer: [u8; 160 * 144],
}

/// Convert a tile data offset t
fn get_tile_data_address(base_address: u16, tile_number: u8) -> u16 {
    if base_address == 0x8800 {
        base_address + ((tile_number as i8 as i16 + 128) as u16 * 16)
    } else {
        base_address + (tile_number as u16 * 16)
    }
}

fn get_pixel_value(tile_lower: u8, tile_upper: u8, pixel_num: u8) -> u8 {
    // Get the two bits that describe this pixel's colour.  A row of 8 pixels are described
    // by two consecutive bytes. The bits however are not consecutive. The first pixel value
    // is described by the most-significant bit of each byte and so forth.
    let p0 = (tile_lower >> (7 - pixel_num)) & 0x1;
    let p1 = (tile_upper >> (7 - pixel_num)) & 0x1;
    (p1 << 1) + p0
}

fn get_palette_color(pixel_value: u8, palette: u8) -> u8 {
    // Get the palette value for this pixel value.
    // Multiply by 2 because hwreg.background_palette is 4  2-bit values. To get the
    // color_value for pixel 00 -> 00,   01 -> 02,  02 -> 04,  03 -> 06.  Mask by 0b11
    // because the color value is two bits.
    (palette >> (pixel_value * 2)) & 0x3
}

impl PPU {
    pub fn new() -> Self {
        Self {
            modeclock: 0,
            bg_color_zero: [false; 160],
            image_buffer: [1; 160 * 144],
        }
    }

    fn set_pixel(&mut self, line: u8, col: u8, value: u8) {
        self.image_buffer[line as usize * 160 + col as usize] = value;
    }

    /// TODO: explain the mode cycle and clocks.
    pub fn step(&mut self, mmu: &mut MMU, cycles: u8) {
        if mmu.ppu.clear_screen {
            self.image_buffer = [1; 160 * 144];
            self.modeclock = 0;
            mmu.ppu.clear_screen = false; // Reset flag.
        }

        let mode = mmu.ppu.mode;

        // Increase the clock by number of cycles being emulated. This will govern what needs
        // to happen next such as changing modes. It is possible that we exceed the number of
        // cycles for the current mode. For that reason, we subtract the expected count from
        // self.modeclock. That allows excessive cycles to be carried over to the next mode.
        self.modeclock += cycles as usize;

        // HBlank. Upon entering this state, we would have successfully drawn a line and are
        // moving on to the next line or vblank.
        // Transitions to: Mode 1 (VBlank) or Mode 2 (OAM Read)
        if mode == 0 && self.modeclock >= 204 {
            self.modeclock -= 204;
            mmu.ppu.line += 1; // Advance 1 line as we're in hblank.

            // At the end of hblank, if on line 143, we've drawn all 144 lines and need to enter
            // vblank. Otherwise go back to mode 2 and loop again.
            if mmu.ppu.line == 143 {
                mmu.ppu.mode = 1;
                // TODO: setting interrupt is more detailed than this.
                mmu.interrupts.intf |= 1; // Set Vblank flag.
            } else {
                mmu.ppu.mode = 2;
            }
        }

        // VBlank. This runs for 10 lines (4560 cycles) and increments hwreg.ly. It is valid
        // for hwreg.ly to be a value of 144 to 152, representing when it is in vblank.
        // Transitions to: Mode 2 (OAM Read)
        if mode == 1 && self.modeclock >= 456 {
            self.modeclock -= 456;

            if mmu.ppu.line == 153 {
                mmu.ppu.mode = 2;
                mmu.ppu.line = 0;
            } else {
                mmu.ppu.line += 1;
            }
        }

        // OAM Read mode.
        // Note that we're not doing OAM read here. We're just simulating the amount of time that
        // the original hardware would take to OAM read. This is necessary to keep all timing
        // in sync. When OAM is needed, it will be read at what's effectively instantaneous speed.
        if mode == 2 && self.modeclock >= 80 {
            self.modeclock -= 80;
            mmu.ppu.mode = 3;
            return;
        }

        // VRAM read mode. End of mode 3 acts as end of scanline.
        if mode == 3 && self.modeclock >= 172 {
            self.modeclock -= 172;
            mmu.ppu.mode = 0;
            self.draw_scanline(mmu);
            return;
        }
    }

    fn draw_scanline(&mut self, mmu: &MMU) {
        if !mmu.ppu.lcd_on {
            return;
        }

        // Reset background priority state.
        self.bg_color_zero = [false; 160];

        self.draw_background_scanline(mmu);
        self.draw_sprites_scanline(mmu);
    }

    /// Modify the current line's buffer with sprite data. Sprite pixels may not draw depending on
    /// OAM settings governing if the sprite is in front of or behind the background.
    /// It's easier to work with `isize` values because we're dealing with a mapping space that
    /// can have negative values (off screen sprites).
    fn draw_sprites_scanline(&mut self, mmu: &MMU) {
        let ppu = &mmu.ppu;
        let line = ppu.line as isize;
        let sprite_y_size = if ppu.sprite_size { 16 } else { 8 } as isize;

        if !ppu.sprite_on {
            return;
        };

        let mut drawn_sprite_count = 0;

        // Walk through all 40 sprites in OAM memory. We need to draw the ones whose coordinates
        // put them within the viewframe.
        for s in 0..40 {
            // Draw max 10 sprites per line.
            if drawn_sprite_count == 10 {
                break;
            }

            // Parse four bytes of data representing the coordinates, sprite number, and flags.
            // The positions are handled as signed integers to allow them to be off the screen.
            // If they remain off the screen when added to the line number or column, they will
            // ultimately not be drawn.
            let oam_address = 0xFE00 + s * 4;
            let y_pos = mmu.rb(oam_address) as isize - 16;
            let x_pos = mmu.rb(oam_address + 1) as isize - 8;
            let sprite_number = mmu.rb(oam_address + 2) as u16;
            let flags = mmu.rb(oam_address + 3);

            let palette = if is_bit_set(flags, 4) {
                ppu.obj_palette_1
            } else {
                ppu.obj_palette_0
            };

            let bg_priority = is_bit_set(flags, 7);
            let y_flip = is_bit_set(flags, 6);
            let x_flip = is_bit_set(flags, 5);

            // Is the sprite not on this line?
            if line < y_pos || line >= y_pos + sprite_y_size {
                continue;
            }

            // Is the sprite off the left or right of the screen?
            if x_pos < -7 || x_pos >= 160 {
                continue;
            }

            drawn_sprite_count += 1;

            // Get the y-coordinate of the current sprite. A sprite is 8 or 16 rows tall.
            // Depending on what line we're rendering, we get one of those lines to draw onto it.
            // If y_flip is true, we invert which line we're getting.
            let sprite_y = if y_flip {
                (sprite_y_size - 1 - (line - y_pos)) as u16
            } else {
                (line - y_pos) as u16
            };

            // Calculate data address of the data for this line of the sprite.
            // Each sprite is 16 bytes, so jump by multiples of 16.
            // Each row is 2 bytes, so jump by 2.
            let sprite_data_address = 0x8000 + (sprite_number * 16) + (sprite_y * 2);

            // Get the sprite data (2 bytes, combined makes a row of 8 pixels).
            let sprite_data_lower = mmu.rb(sprite_data_address);
            let sprite_data_upper = mmu.rb(sprite_data_address + 1);

            // Walk through each pixel to be drawn.
            for p in 0..8isize {
                // Is this specific pixel not on the screen?
                if x_pos + p < 0 || x_pos + p >= 160 {
                    continue;
                }

                // Don't draw if hiding under the background.
                if mmu.ppu.window_bg_on && bg_priority && !self.bg_color_zero[x_pos as usize] {
                    continue;
                }

                // Number of pixel (0-7) of this row of the sprite. Might be horizontally flipped.
                let pixel_num = if x_flip { 7 - p } else { p };

                let pixel_value =
                    get_pixel_value(sprite_data_lower, sprite_data_upper, pixel_num as u8);

                let color = get_palette_color(pixel_value, palette);

                let col = x_pos + p;
                self.set_pixel(line as u8, col as u8, color);
            }
        }
    }

    /// Draw a single scanline by iterating through a line of pixels and getting pixel data from
    /// the relevant tiles. Only a subset of the 256x256 scene is displayed, so we are not always
    /// drawing complete tiles. There's also wrap-around possible.
    fn draw_background_scanline(&mut self, mmu: &MMU) {
        // if mmu.ppu.window_bg_on {
        //     return;
        // }

        let ppu = &mmu.ppu;

        // Flags determine which of the two tilemaps and tiledata tables we use.
        let is_tiledata_low = ppu.tile_data_table;
        let is_tilemap_high = ppu.bg_tilemap;

        // Use the LCDC hardware register to determine which of the two tilemap spaces we are
        // utilizing. They both behave the same in all ways.
        let tilemap_address = if is_tilemap_high { 0x9C00 } else { 0x9800 };

        // Use the LCDC hardware register to determine which of the two tile data spaces in VRAM we
        // are utilizing. The upper tiledata table beginning at 0x8800 needs to be accessed
        // with a signed value, indexing on 0x9000.
        let tiledata_base_address = if is_tiledata_low { 0x8000 } else { 0x8800 };

        // Scroll offsets.  The tile maps represent a 256x256 scene of pixels. We only want to
        // render a 160x144 viewport of it.
        let scy = ppu.scy;
        let scx = ppu.scx;

        // We want to iterate through 160 pixels to draw one scanline.
        for col in 0..160u8 {
            // Calculate tilemap pixel indexes by adding the current pixel x,y with the scroll
            // register values. This accounts for the viewport we want to draw not being the same
            // as the 256x256 tilemap scene.
            let x = col.wrapping_add(scx);
            let y = ppu.line.wrapping_add(scy);

            // There are 1024 tiles mapped in a 32x32 grid of 8x8 pixel tiles. The 1024 tiles are
            // described in one of the two tile maps as a row-major array. To get the tile number
            // we divide row and column of the pixel we're looking up by the number of pixels per
            // tile. We then walk the row-major grid to get the single tile number.
            let tile_row_num = y / 8;
            let tile_col_num = x / 8;
            let tile_number = tile_row_num as u16 * 32 + tile_col_num as u16;

            // We can then look up the tile's data address by accessing the tile map at the offset
            // address + tile number. To find the address of the correct tile, multiply this address
            // by 16 (the size of each whole tile's worth of data) and add (or subtract) that to
            // the tiledata_base_address.
            // // If we are accessing TILEDATA_1, we need to access it with a signed offset.
            let tile_data_number = mmu.rb(tilemap_address + tile_number);
            let tile_data_address = get_tile_data_address(tiledata_base_address, tile_data_number);

            // Get the pixel coordinates in the local 8x8 tile.
            let pixel_row_num = y % 8;
            let pixel_col_num = x % 8;

            // While tile_data_address is the address of the beginning of the entire tile, the
            // tile_row_address is the address that the specific row of data where this pixel
            // is found. We multiply by 2 because every row of 8 pixels is 2 bytes of data.
            let tile_row_index = tile_data_address + (pixel_row_num as u16 * 2);
            let tile_data_lower = mmu.rb(tile_row_index);
            let tile_data_upper = mmu.rb(tile_row_index + 1);

            let pixel_value = get_pixel_value(tile_data_lower, tile_data_upper, pixel_col_num);
            let color = get_palette_color(pixel_value, ppu.background_palette);

            // Set background priority.
            self.bg_color_zero[col as usize] = pixel_value == 0;

            // Update the image buffer with this pixel value. Given a well-behaved main loop should
            // iterate through every pixel, there is no need to clear the previous buffer data.
            self.set_pixel(ppu.line, x, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tile_data_address() {
        // low tile data, access as unsigned.
        let result = get_tile_data_address(0x8000, 0xFF);
        assert_eq!(result, 0x8FF0);

        // high tile data, access as signed.
        // 0x00 would be on the middle value (0x9000)
        let result = get_tile_data_address(0x8800, 0x00);
        assert_eq!(result, 0x9000);

        // 0b10000000
        let result = get_tile_data_address(0x8800, 0x80);
        assert_eq!(result, 0x8800);
    }
}
