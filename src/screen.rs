pub struct Screen {
    _sdl_canvas: sdl2::render::Canvas<sdl2::video::Window>,
    _scale_factor: u32,
}

impl Screen {
    const DMG_WIDTH: u32 = 160;
    const DMG_HEIGHT: u32 = 144;

    pub fn new(context: &sdl2::Sdl, scale_factor: u32) -> Result<Self, String> {
        let video_subsys = context.video()?;

        let window = video_subsys
            .window(
                "title: CHIP8",
                Self::DMG_WIDTH * scale_factor,
                Self::DMG_HEIGHT * scale_factor,
            )
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let c = window.into_canvas().build().map_err(|e| e.to_string())?;

        Ok(Self {
            _sdl_canvas: c,
            _scale_factor: scale_factor,
        })
    }
}
