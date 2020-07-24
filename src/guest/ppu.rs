pub struct PPU {
    modeclock: usize, // Current clock step representing where the PPU is in its processing cycle.
    pub image_buffer: [u8; 160 * 144],
}
use super::mmu::{MMU, TILEDATA_0, TILEDATA_1, TILEMAP_0, TILEMAP_1};

impl PPU {
    pub fn new() -> Self {
        Self {
            modeclock: 0,
            image_buffer: [1; 160 * 144],
        }
    }

    /// TODO: explain the mode cycle and clocks.
    pub fn step(&mut self, mmu: &mut MMU, cycles: u8) {
        let mode = mmu.hwreg.mode;

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
            mmu.hwreg.mode = 3;
            return;
        }

        // VRAM read mode. End of mode 3 acts as end of scanline.
        if mode == 3 && self.modeclock >= 172 {
            self.modeclock -= 172;
            mmu.hwreg.mode = 0;
            self.draw_scanline(mmu);
            return;
        }

        // HBlank. Upon entering this state, we would have successfully drawn a line and are
        // moving on to the next line or vblank.
        if mode == 0 && self.modeclock >= 204 {
            self.modeclock -= 204;
            mmu.hwreg.line += 1; // Advance 1 line as we're in hblank.

            // At the end of hblank, if on line 143, we've drawn all 144 lines and need to enter
            // vblank. Otherwise go back to mode 2 and loop again.
            if mmu.hwreg.line == 143 {
                mmu.hwreg.mode = 1;
            } else {
                mmu.hwreg.mode = 2;
            }
        }

        // VBlank. This runs for 10 lines (4560 cycles) and does increment hwreg.ly. It is valid
        // for hwreg.ly to be a value of 144 to 152, representing when it is in vblank.
        if mode == 1 && self.modeclock >= 456 {
            self.modeclock -= 456;

            if mmu.hwreg.line == 153 {
                mmu.hwreg.mode = 2;
                mmu.hwreg.line = 0;
            } else {
                mmu.hwreg.line += 1;
            }
        }
    }

    /// TODO better detail.
    fn draw_scanline(&mut self, mmu: &MMU) {
        let line = mmu.hwreg.line;

        // Which of the two tilemaps are we utilizing?
        let tilemap_address = if mmu.hwreg.bg_tilemap {
            TILEMAP_1
        } else {
            TILEMAP_0
        };

        let tiledata_address = if mmu.hwreg.tile_data_table {
            TILEDATA_0
        } else {
            TILEDATA_1
        };

        let scy = mmu.hwreg.scy; // Scroll-y offset.
        let scx = 0u8; // TODO: This should be a hwreg. Not used in the vertical scroll intro.

        // We want to iterate through 160 pixels to draw one scanline.
        for x in 0..160u8 {
            let index_x = x.wrapping_add(scx); // Pixel's x in 256x256 scene with wraparound.
            let tile_col_num = index_x / 8; // 32x32 tile x coord.

            let index_y = line.wrapping_add(scy); // Pixel's y in 256x256 scene with wraparound.
            let tile_row_num = index_y / 8; // 32x32 tile y coord.

            // At this point we have which tile in the 32x32 tilemap the pixel we want to draw
            // TODO: more exposition.

            // Get what pixel in this 8*8 tile we're drawing.
            let pixel_row_num = index_x % 8; // Which bit in the tile byte is this pixel?
            let pixel_col_num = index_y % 8;

            // Thre are 1024 tiles numbered 0-1023.
            let tile_num = tile_row_num as u16 * 32 + tile_col_num as u16;
            let tile_address = tiledata_address + tile_num; // First pixel in tile.

            // Multiply by 2 because each row is two bytes.
            let tile_row_index = tile_address + (pixel_row_num as u16 * 2);
            let tile_data_low = mmu.rb(tile_row_index);
            let tile_data_high = mmu.rb(tile_row_index + 1);

            // Get pixel bits.
            let p0 = (tile_data_low >> pixel_col_num) & 0x1;
            let p1 = (tile_data_high >> pixel_col_num) & 0x1;

            let pvalue = p1 << 1 + p0;

            self.image_buffer[line as usize * 160 + x as usize] = pvalue;
        }
    }
}
