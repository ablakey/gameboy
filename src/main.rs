mod emulator;
mod host;
use emulator::{CPU, MMU};
use host::{init_debugger, Input, InputEvent, Screen};
use sdl2;

fn main() {
    init_debugger();
    let emulator = Emulator::new();

    match emulator {
        Ok(mut e) => e.run_forever(),
        Err(e) => panic!("Could not launch emulator. {}", e),
    }
}

struct Emulator {
    cpu: CPU,
    mmu: MMU,
    input: Input,
    _screen: Screen,
    is_paused: bool,
}

impl Emulator {
    fn new() -> Result<Self, String> {
        // SDL-based host: graphics, sound, audio.
        let sdl_context = sdl2::init()?;
        let input = Input::new(&sdl_context)?;
        let _screen = Screen::new(&sdl_context, 4)?;
        Ok(Self {
            cpu: CPU::new(),
            mmu: MMU::new(),
            input,
            is_paused: false,
            _screen,
        })
    }

    pub fn run_forever(&mut self) {
        'program: loop {
            // Handle program I/O (events that affect the emulator).
            match self.input.get_event() {
                InputEvent::Exit => break 'program,
                InputEvent::ToggleRun => self.is_paused = !self.is_paused,
                InputEvent::Tick => {
                    self.step();
                }
                _ => (),
            }

            if !self.is_paused {
                self.step();
            }

            // Regulate the hot loop a little. Calibrate later.
            // sleep(Duration::new(0, 2_000_000 as u32))
        }
    }

    pub fn step(&mut self) {
        self.cpu.step(&mut self.mmu);
    }
}
