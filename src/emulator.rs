use super::guest::{CPU, MMU, PPU};
use super::host::{Input, InputEvent, Screen};
use sdl2;

pub struct Emulator {
    cpu: CPU,
    ppu: PPU,
    mmu: MMU,
    input: Input,
    screen: Screen,
    is_paused: bool,
}

impl Emulator {
    pub fn new(cartridge_path: Option<&String>) -> Result<Self, String> {
        // SDL-based host: graphics, sound, audio.
        let sdl_context = sdl2::init()?;
        let input = Input::new(&sdl_context)?;
        let screen = Screen::new(&sdl_context, 4)?;
        Ok(Self {
            cpu: CPU::new(),
            mmu: MMU::new(cartridge_path),
            ppu: PPU::new(),
            input,
            is_paused: false,
            screen,
        })
    }

    pub fn run_forever(&mut self) {
        'program: loop {
            // Handle program I/O (events that affect the emulator). This needs to be
            match self.input.get_event() {
                InputEvent::Exit => break 'program,
                InputEvent::ToggleRun => self.is_paused = !self.is_paused,
                _ => (),
            }

            self.emulate_frame();
        }
    }

    fn emulate_frame(&mut self) {
        let mut cycle_count: usize = 0;
        'frame: loop {
            // TODO: this loop will expand to step one line at a time through the CPU, PPU, APU.
            let cycles = self.cpu.step(&mut self.mmu);
            self.ppu.step(&mut self.mmu, cycles);
            cycle_count += cycles as usize;

            // 4Mhz cpu at 60fps.
            if cycle_count >= (4194304 / 60) {
                break 'frame;
            }
        }

        // Draw the frame.  Note that vsync is enabled so this is ultimately what governs the
        // rate of this emulator. The SDL drawing routine will block for the next frame. This also
        // means that if the framerate goverened by v-sync isn't 60fps, this emulator won't work
        // right. That's okay for my purposes. Check out some other emulators for other ways to
        // handle this.  the rboy Rust emulator uses a thread to ping on a regular interval. The
        // main loop can block on awaiting that ping. There's probably also a really smart way
        // to handle it using async/await.
        self.screen.update(&self.ppu.image_buffer);
    }
}
