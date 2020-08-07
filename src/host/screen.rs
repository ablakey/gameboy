use sdl2;

pub struct Screen {
    sdl_canvas: sdl2::render::Canvas<sdl2::video::Window>,
}

impl Screen {
    const DMG_WIDTH: usize = 160;
    const DMG_HEIGHT: usize = 144;

    const PALETTE_HIGH: (u8, u8, u8) = (155, 188, 15); // #9bbc0f
    const PALETTE_MED: (u8, u8, u8) = (139, 172, 15); // #8bac0f
    const PALETTE_LOW: (u8, u8, u8) = (48, 98, 48); // #306230
    const PALETTE_OFF: (u8, u8, u8) = (15, 56, 15); // #0f380f

    pub fn new(context: &sdl2::Sdl, scale_factor: usize) -> Result<Self, String> {
        let video_subsys = context.video()?;

        let window = video_subsys
            .window(
                "Blakey's Gameboy",
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

        Ok(Self { sdl_canvas: canvas })
    }

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
