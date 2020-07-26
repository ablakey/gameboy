use sdl2;

pub struct Screen {
    sdl_canvas: sdl2::render::Canvas<sdl2::video::Window>,
    scale_factor: usize,
}

impl Screen {
    const DMG_WIDTH: usize = 160;
    const DMG_HEIGHT: usize = 144;

    const PALETTE_HIGH: (u8, u8, u8) = (255, 255, 255);
    const PALETTE_MED: (u8, u8, u8) = (192, 192, 192);
    const PALETTE_LOW: (u8, u8, u8) = (100, 100, 100);
    const PALETTE_OFF: (u8, u8, u8) = (0, 0, 0);

    // TODO: need 4 colors: off, 33%, 66%, 100%
    const BG_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(0, 0, 0);
    const PIXEL_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(255, 255, 255);

    pub fn new(context: &sdl2::Sdl, scale_factor: usize) -> Result<Self, String> {
        let video_subsys = context.video()?;

        let window = video_subsys
            .window(
                "title: Gameboy",
                (Self::DMG_WIDTH * scale_factor) as u32,
                (Self::DMG_HEIGHT * scale_factor) as u32,
            )
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Self {
            sdl_canvas: canvas,
            scale_factor,
        })
    }

    /// Iterate through all pixels in buffer and draw only those that are set active (b&w).
    /// The screen is first blanked, then all pixels in buffer are evaluated for being active.
    /// The remaining pixels are drawn as filled rects, scaled by scale_factor.
    // pub fn draw(&mut self, &buffer: &[u8; Self::DMG_WIDTH * Self::DMG_HEIGHT]) {
    //     let rects: Vec<sdl2::rect::Rect> = buffer
    //         .iter()
    //         .enumerate()
    //         .filter(|(_, &x)| x > 0)
    //         .map(|(n, _)| {
    //             // Row-major, so we divide and modulo by width to get row and column number.
    //             let row = n / Self::DMG_WIDTH as usize;
    //             let col = n % Self::DMG_WIDTH as usize;

    //             return sdl2::rect::Rect::new(
    //                 (col * self.scale_factor as usize) as i32,
    //                 (row * self.scale_factor as usize) as i32,
    //                 self.scale_factor as u32,
    //                 self.scale_factor as u32,
    //             );
    //         })
    //         .collect();

    //     self.sdl_canvas.set_draw_color(Self::BG_COLOR);
    //     self.sdl_canvas.clear();

    //     self.sdl_canvas.set_draw_color(Self::PIXEL_COLOR);
    //     self.sdl_canvas.fill_rects(&rects).unwrap();
    //     self.sdl_canvas.present();
    // }

    /// Update the screen using a buffer of pixel values.
    /// Given the DMG-01 has only four possible colours, the pixel values will be 0-3.
    pub fn update(&mut self, &buffer: &[u8; Self::DMG_WIDTH * Self::DMG_HEIGHT]) {
        let mut texture_data = [0u8; Self::DMG_WIDTH * Self::DMG_HEIGHT * 3];

        for (index, pixel) in buffer.iter().enumerate() {
            let (r, g, b) = match pixel {
                0 => Self::PALETTE_HIGH,
                1 => Self::PALETTE_MED,
                2 => Self::PALETTE_LOW,
                3 => Self::PALETTE_OFF,
                _ => panic!("Passed a non-valid value to Screen.update: {}", pixel),
            };

            // Populate the texture data's R,G,B.
            texture_data[index * 3] = r;
            texture_data[index * 3 + 1] = g;
            texture_data[index * 3 + 2] = b;
        }

        // Create the texture.
        let creator = self.sdl_canvas.texture_creator();
        let mut texture = creator
            .create_texture(
                sdl2::pixels::PixelFormatEnum::RGB24,
                sdl2::render::TextureAccess::Static,
                Self::DMG_WIDTH as u32,
                Self::DMG_HEIGHT as u32,
            )
            .unwrap();

        texture
            .update(None, &texture_data, Self::DMG_WIDTH * 3)
            .unwrap();

        self.sdl_canvas.copy(&texture, None, None).unwrap();
        self.sdl_canvas.present();
    }
}
