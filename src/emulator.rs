use crate::guest::systems::{Gamepad, Timer, CPU, PPU};
use crate::guest::MMU;
use crate::host::{Audio, Input, InputEvent, Screen};
use sdl2;

pub const CPU_FREQ: usize = 4194304; // 4MHz for DMG-01.
pub const DIVIDER_FREQ: usize = CPU_FREQ / 16384; // Divider always runs at 16KHz.
const FRAMERATE: usize = 60;

pub struct Emulator {
    // Guest components.
    cpu: CPU,
    ppu: PPU,
    mmu: MMU,
    gamepad: Gamepad,
    timer: Timer,
    // Host components.
    input: Input,
    screen: Screen,
    audio: Audio,
}

impl Emulator {
    pub fn new(cartridge_path: Option<&String>, use_bootrom: bool) -> Result<Self, String> {
        // SDL-based host: graphics, sound, audio.
        let sdl_context = sdl2::init()?;
        let input = Input::new(&sdl_context)?;
        let screen = Screen::new(&sdl_context, 4)?;
        let audio = Audio::new(&sdl_context)?;
        Ok(Self {
            cpu: CPU::new(),
            mmu: MMU::new(cartridge_path, use_bootrom),
            ppu: PPU::new(),
            timer: Timer::new(),
            gamepad: Gamepad::new(),
            input,
            audio,
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

    /// Emulate one whole frame work of CPU, PPU, Timer work. Given 60fps, 1 frame is 1/60 of the
    /// CPU clock speed worth of work:
    fn emulate_frame(&mut self) {
        let mmu = &mut self.mmu;
        let mut cycle_count: usize = 0;

        // Update gamepad input state. Do this at 60hz to save on CPU.
        let gamepad_state = self.input.get_gamepad_state();
        self.gamepad.update_state(gamepad_state);

        'frame: loop {
            // Gamepad step.
            self.gamepad.step(mmu);

            // CPU step.
            let cycles = self.cpu.step(mmu);

            // Timer step.
            self.timer.step(mmu, cycles);

            // PPU step.
            self.ppu.step(mmu, cycles);

            // 4Mhz cpu at 60fps.
            cycle_count += cycles as usize;
            if cycle_count >= (CPU_FREQ / FRAMERATE) {
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

        // self.audio.
    }
}
