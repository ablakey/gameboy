use crate::guest::systems::{Gamepad, CPU, PPU};
use crate::guest::MMU;
use crate::host::{Input, InputEvent, Screen};
use sdl2;

pub struct Emulator {
    // Guest components.
    cpu: CPU,
    ppu: PPU,
    mmu: MMU,
    gamepad: Gamepad,

    // Host components.
    input: Input,
    screen: Screen,
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
            gamepad: Gamepad::new(),
            input,
            screen,
        })
    }

    pub fn run_forever(&mut self) {
        'program: loop {
            // Handle program I/O (events that affect the emulator). This needs to be
            match self.input.get_event() {
                InputEvent::Exit => break 'program,
                InputEvent::Panic => panic!("Panic caused by user."),
                _ => (),
            }
            self.emulate_frame();
        }
    }

    pub fn dump_state(&self) {
        self.mmu.dump_state();
    }

    fn emulate_frame(&mut self) {
        let mmu = &mut self.mmu;
        let mut cycle_count: usize = 0;

        // Update gamepad input state. Do this at 60hz to save on CPU.
        let gamepad_state = self.input.get_gamepad_state();
        self.gamepad.update_state(mmu, gamepad_state);

        'frame: loop {
            // Gamepad step.
            self.gamepad.step(mmu);

            // CPU step.
            let cycles = self.cpu.step(mmu);

            // PPU step.
            self.ppu.step(mmu, cycles);
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
