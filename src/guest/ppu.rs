pub struct PPU {
    modeclock: usize, // Current clock step representing where the PPU is in its processing cycle.
    pub image_buffer: [u8; 160 * 144],
}
use super::mmu::{MMU, TILEDATA_0, TILEDATA_1, TILEMAP_0, TILEMAP_1};

/// Convert a tile data offset t
fn get_tile_data_address(base_address: u16, tile_number: u8) -> u16 {
    if base_address == TILEDATA_1 {
        base_address + ((tile_number as i8 as i16 + 128) as u16 * 16)
    } else {
        base_address + (tile_number as u16 * 16)
    }
}

impl PPU {
    pub fn new() -> Self {
        Self {
            modeclock: 0,
            image_buffer: [1; 160 * 144],
        }
    }

    /// TODO: explain the mode cycle and clocks.
    pub fn step(&mut self, mmu: &mut MMU, cycles: u8) {
        let mode = mmu.hwreg.ppu.mode;

        // Increase the clock by number of cycles being emulated. This will govern what needs
        // to happen next such as changing modes. It is possible that we exceed the number of
        // cycles for the current mode. For that reason, we subtract the expected count from
        // self.modeclock. That allows excessive cycles to be carried over to the next mode.
        self.modeclock += cycles as usize;

        // OAM Read mode.
        // Note that we're not doing OAM read here. We're just simulating the amount of time that
        // the original hardware would take to OAM read. This is necessary to keep all timing
        // in sync. When OAM is needed, it will be read at what's effectively instantaneous speed.
        if mode == 2 && self.modeclock >= 80 {
            self.modeclock -= 80;
            mmu.hwreg.ppu.mode = 3;
            return;
        }

        // VRAM read mode. End of mode 3 acts as end of scanline.
        if mode == 3 && self.modeclock >= 172 {
            self.modeclock -= 172;
            mmu.hwreg.ppu.mode = 0;
            self.draw_scanline(mmu);
            return;
        }

        // HBlank. Upon entering this state, we would have successfully drawn a line and are
        // moving on to the next line or vblank.
        if mode == 0 && self.modeclock >= 204 {
            self.modeclock -= 204;
            mmu.hwreg.ppu.line += 1; // Advance 1 line as we're in hblank.

            // At the end of hblank, if on line 143, we've drawn all 144 lines and need to enter
            // vblank. Otherwise go back to mode 2 and loop again.
            if mmu.hwreg.ppu.line == 143 {
                mmu.hwreg.ppu.mode = 1;
            } else {
                mmu.hwreg.ppu.mode = 2;
            }
        }

        // VBlank. This runs for 10 lines (4560 cycles) and does increment hwreg.ly. It is valid
        // for hwreg.ly to be a value of 144 to 152, representing when it is in vblank.
        if mode == 1 && self.modeclock >= 456 {
            self.modeclock -= 456;

            if mmu.hwreg.ppu.line == 153 {
                mmu.hwreg.ppu.mode = 2;
                mmu.hwreg.ppu.line = 0;
            } else {
                mmu.hwreg.ppu.line += 1;
            }
        }
    }

    fn draw_scanline(&mut self, mmu: &MMU) {
        if !mmu.hwreg.ppu.lcd_on {
            return;
        }

        self.draw_background_scanline(mmu);
    }

    /// Draw a single scanline by iterating through a line of pixels and getting pixel data.
    fn draw_background_scanline(&mut self, mmu: &MMU) {
        let ppureg = &mmu.hwreg.ppu;

        // Use the LCDC hardware register to determine which of the two tilemap spaces we are
        // utilizing. They both behave the same in all ways.
        let tilemap_address = if ppureg.bg_tilemap {
            TILEMAP_1
        } else {
            TILEMAP_0
        };

        // Use the LCDC hardware register to determine which of the two tile data spaces in VRAM we
        // are utilizing. The upper tiledata table beginning at 0x8800 needs to be accessed
        // with a signed value, indexing on 0x9000.
        let tiledata_base_address = if ppureg.tile_data_table {
            TILEDATA_0
        } else {
            TILEDATA_1
        };

        // Scroll offsets.  The tile maps represent a 256x256 scene of pixels. We only want to
        // render a 160x144 viewport of it.
        let scy = ppureg.scy;
        let scx = ppureg.scx;

        // We want to iterate through 160 pixels to draw one scanline.
        for x in 0..160u8 {
            // Calculate tilemap pixel indexes by adding the current pixel x,y with the scroll
            // register values. This accounts for the viewport we want to draw not being the same
            // as the 256x256 tilemap scene.
            let x = x.wrapping_add(scx);
            let y = ppureg.line.wrapping_add(scy);

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

            // Get the two bits that describe this pixel's colour.  A row of 8 pixels are described
            // by two consecutive bytes. The bits however are not consecutive. The first pixel value
            // is described by the most-significant bit of each byte and so forth.
            let p0 = (tile_data_lower >> (7 - pixel_col_num)) & 0x1;
            let p1 = (tile_data_upper >> (7 - pixel_col_num)) & 0x1;
            let pixel_value = (p1 << 1) + p0;

            // Get the palette value for this pixel value.
            // Multiply by 2 because hwreg.background_palette is 4  2-bit values. To get the
            // color_value for pixel 00 -> 00,   01 -> 02,  02 -> 04,  03 -> 06.  Mask by 0b11
            // because the color value is two bits.
            let color_value = (ppureg.background_palette >> (pixel_value * 2)) & 0x3;

            // Update the image buffer with this pixel value. Given a well-behaved main loop should
            // iterate through every pixel, there is no need to clear the previous buffer data.
            self.image_buffer[ppureg.line as usize * 160 + x as usize] = color_value;
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
