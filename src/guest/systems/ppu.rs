use super::super::mmu::is_bit_set;
use super::MMU;

/// Given MMU state, coordinates, and the address to the current tilemap, get the pixel value.
fn get_tile_pixel(mmu: &MMU, x: u8, y: u8, tilemap_address: u16) -> u8 {
    // Use the LCDC hardware register to determine which of the two tile data spaces in VRAM we
    // are utilizing. The upper tiledata table beginning at 0x8800 needs to be accessed
    // with a signed value, indexing on 0x9000.
    let tiledata_base_address = if mmu.ppu.tile_data_table {
        0x8000
    } else {
        0x8800
    };

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
    // If we are accessing TILEDATA_1, we need to access it with a signed offset.
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

    get_pixel(tile_data_lower, tile_data_upper, pixel_col_num)
}

/// Get the address for tile data, given the base address and a tile number. This way to access
/// changes based on base address, as one of the two address spaces needs to be indexed
/// with a singed integer.
fn get_tile_data_address(base_address: u16, tile_number: u8) -> u16 {
    if base_address == 0x8800 {
        base_address + ((tile_number as i8 as i16 + 128) as u16 * 16)
    } else {
        base_address + (tile_number as u16 * 16)
    }
}

/// Get the two bits that describe this pixel's colour.  A row of 8 pixels are described
/// by two consecutive bytes. The bits however are not consecutive. The first pixel value
/// is described by the most-significant bit of each byte and so forth.
fn get_pixel(tile_lower: u8, tile_upper: u8, pixel_num: u8) -> u8 {
    let p0 = (tile_lower >> (7 - pixel_num)) & 0x1;
    let p1 = (tile_upper >> (7 - pixel_num)) & 0x1;
    (p1 << 1) + p0
}

pub struct PPU {
    modeclock: usize, // Current clock step representing where the PPU is in its processing cycle.
    pub bg_color_zero: [bool; 160], // tracks which pixels in a row have background = 0.
    pub image_buffer: [u8; 160 * 144],
}

impl PPU {
    pub fn new() -> Self {
        Self {
            modeclock: 0,
            bg_color_zero: [false; 160],
            image_buffer: [0; 160 * 144],
        }
    }

    fn draw_pixel(&mut self, line: u8, col: u8, value: u8) {
        self.image_buffer[line as usize * 160 + col as usize] = value;
    }

    /// TODO: explain the mode cycle and clocks.
    pub fn step(&mut self, mmu: &mut MMU, cycles: u8) {
        // The screen might be cleared entirely because the PPU's state has it shut off. Note that
        // line and mode were also set to 0 (in the ppu )
        if mmu.ppu.clear_screen {
            self.image_buffer = [0; 160 * 144];
            self.modeclock = 0;
            mmu.ppu.line = 0;
            mmu.ppu.mode = 0;
            mmu.ppu.clear_screen = false; // Reset flag.
        }

        let mode = mmu.ppu.mode;

        // Increase the clock by number of cycles being emulated. This will govern what needs
        // to happen next such as changing modes. It is possible that we exceed the number of
        // cycles for the current mode. For that reason, we subtract the expected count from
        // self.modeclock. That allows excessive cycles to be carried over to the next mode.
        self.modeclock += cycles as usize;

        if self.modeclock >= 456 {
            self.modeclock -= 456;
            mmu.ppu.line = (mmu.ppu.line + 1) % 154;
            mmu.check_lyc_interrupt();

            // VBlank line.
            if mmu.ppu.line >= 144 && mode != 1 {
                mmu.ppu.mode = 1;

                // LCDC Status interrupt entering mode 1?
                if mmu.ppu.mode1_int_enable {
                    mmu.interrupts.intf |= 0x02;
                }
                mmu.interrupts.intf |= 0x01; // Set Vblank interrupt flag.
            }
        }

        // Only handle mode changes if we're in a normal line.
        if mmu.ppu.line < 144 {
            // Determine if mode should change and interrupt should be set.
            let change_mode = match self.modeclock {
                0..=80 if mode != 2 => Some((2, mmu.ppu.mode2_int_enable)),
                81..=252 if mode != 3 => Some((3, false)),
                253..=455 if mode != 0 => Some((0, mmu.ppu.mode0_int_enable)),
                _ => None,
            };

            // Change mode and possibly set interrupt.
            match change_mode {
                Some((next_mode, set_interrupt)) => {
                    mmu.ppu.mode = next_mode;
                    if set_interrupt {
                        mmu.interrupts.intf |= 0x02;
                    }

                    // Draw the line only when mode switches to 0.
                    if next_mode == 0 {
                        self.draw_scanline(mmu);
                    }
                }
                None => {}
            }
        }
    }

    fn draw_scanline(&mut self, mmu: &MMU) {
        if !mmu.ppu.lcd_on {
            return;
        }

        // Reset background priority state.
        self.bg_color_zero = [false; 160];

        self.draw_background_scanline(mmu);
        self.draw_window_scanline(mmu);
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

        // Walk through 40 sprites in OAM memory and collect the first 10 that draw on this line.
        let mut sprites_to_draw: Vec<u8> = Vec::new();

        for s in 0..40 {
            if sprites_to_draw.len() == 10 {
                break;
            }

            // Get the sprite. Does it get drawn on this line and is on screen?
            let oam_address = 0xFE00 + s * 4;
            let y_pos = mmu.rb(oam_address) as isize - 16;
            let x_pos = mmu.rb(oam_address + 1) as isize - 8;

            if line < y_pos || line >= y_pos + sprite_y_size || x_pos < -7 || x_pos >= 160 {
                continue;
            }

            sprites_to_draw.push(s as u8);
        }

        // There's now up to 10 sprites (stored as simple OAM sprite index numbers) to be drawn.
        // iterate this list in reverse to draw, because the earlier sprites in OAM get priority.
        // Note: we already verified that these sprites should be drawn.
        for &s in sprites_to_draw.iter().rev() {
            // Parse four bytes of data representing the coordinates, sprite number, and flags.
            // The positions are handled as signed integers to allow them to be off the screen.
            // If they remain off the screen when added to the line number or column, they will
            // ultimately not be drawn.
            let oam_address = 0xFE00 + s as u16 * 4;
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
                let pixel_value = get_pixel(sprite_data_lower, sprite_data_upper, pixel_num as u8);
                let color = (palette >> (pixel_value * 2)) & 0x3;

                self.draw_pixel(line as u8, (x_pos + p) as u8, color);
            }
        }
    }

    /// Draw the window. This is very similar to the background but is implemented separately to
    /// make the code more understandable. The cost is a bit of repetition and some unnecessary
    /// drawing of background pixels that immediately get covered  up by the window.
    fn draw_window_scanline(&mut self, mmu: &MMU) {
        let ppu = &mmu.ppu;

        if !ppu.window_on {
            return;
        }

        // The y coord of the top-left of this current line of the window.
        let win_y = ppu.line as isize - ppu.win_y as isize;

        // Current line is not
        if win_y < 0 {
            return;
        }

        let tilemap_address = if ppu.window_tilemap { 0x9C00 } else { 0x9800 };

        for x in 0..160u8 {
            let win_x = 0 - (ppu.win_x as isize - 7) + x as isize;

            // The window draws off the screen for this pixel.
            if win_x < 0 || win_x >= 160 {
                continue;
            }

            let pixel = get_tile_pixel(mmu, win_x as u8, win_y as u8, tilemap_address);

            self.draw_pixel(ppu.line, x, pixel);
        }
    }

    /// Draw a single scanline by iterating through a line of pixels and getting pixel data from
    /// the relevant tiles. Only a subset of the 256x256 scene is displayed, so we are not always
    /// drawing complete tiles. There's also wrap-around possible.
    fn draw_background_scanline(&mut self, mmu: &MMU) {
        let ppu = &mmu.ppu;

        // If LCDC0 (window and bg on) is false, don't draw anything.
        if !ppu.window_bg_on {
            return;
        }

        // Use the LCDC hardware register to determine which of the two tilemap spaces we are
        // utilizing. They both behave the same in all ways.
        let tilemap_address = if ppu.bg_tilemap { 0x9C00 } else { 0x9800 };

        // We want to iterate through 160 pixels to draw one scanline.
        for col in 0..160u8 {
            // Calculate tilemap pixel indexes by adding the current pixel x,y with the scroll
            // register values. This accounts for the viewport we want to draw not being the same
            // as the 256x256 tilemap scene.
            let x = col.wrapping_add(ppu.scx);
            let y = ppu.line.wrapping_add(ppu.scy);

            let pixel_value = get_tile_pixel(mmu, x, y, tilemap_address);

            let color = (ppu.background_palette >> (pixel_value * 2)) & 0x3;

            // Set background priority.
            self.bg_color_zero[col as usize] = pixel_value == 0;

            // Update the image buffer with this pixel value. Given a well-behaved main loop should
            // iterate through every pixel, there is no need to clear the previous buffer data.
            self.draw_pixel(ppu.line, col, color);
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
