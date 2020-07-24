pub struct Screen {
    sdl_canvas: sdl2::render::Canvas<sdl2::video::Window>,
    scale_factor: usize,
}

impl Screen {
    const DMG_WIDTH: usize = 160;
    const DMG_HEIGHT: usize = 144;
    // TODO: need 4 colors: off, 33%, 66%, 100%
    const BG_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(0, 0, 0);
    const PIXEL_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(255, 255, 255);

    pub fn new(context: &sdl2::Sdl, scale_factor: usize) -> Result<Self, String> {
        let video_subsys = context.video()?;

        let window = video_subsys
            .window(
                "title: CHIP8",
                (Self::DMG_WIDTH * scale_factor) as u32,
                (Self::DMG_HEIGHT * scale_factor) as u32,
            )
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let c = window.into_canvas().build().map_err(|e| e.to_string())?;

        Ok(Self {
            sdl_canvas: c,
            scale_factor,
        })
    }

    /// Iterate through all pixels in buffer and draw only those that are set active (b&w).
    /// The screen is first blanked, then all pixels in buffer are evaluated for being active.
    /// The remaining pixels are drawn as filled rects, scaled by scale_factor.
    pub fn draw(&mut self, &buffer: &[u8; Self::DMG_WIDTH * Self::DMG_HEIGHT]) {
        let rects: Vec<sdl2::rect::Rect> = buffer
            .iter()
            .enumerate()
            .filter(|(_, &x)| x > 0)
            .map(|(n, _)| {
                // Row-major, so we divide and modulo by width to get row and column number.
                let row = n / Self::DMG_WIDTH as usize;
                let col = n % Self::DMG_WIDTH as usize;

                return sdl2::rect::Rect::new(
                    (col * self.scale_factor as usize) as i32,
                    (row * self.scale_factor as usize) as i32,
                    self.scale_factor as u32,
                    self.scale_factor as u32,
                );
            })
            .collect();

        self.sdl_canvas.set_draw_color(Self::BG_COLOR);
        self.sdl_canvas.clear();

        self.sdl_canvas.set_draw_color(Self::PIXEL_COLOR);
        self.sdl_canvas.fill_rects(&rects).unwrap();
        self.sdl_canvas.present();
    }
}
