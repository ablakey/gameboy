use super::guest::{CPU, MMU};
use super::host::{Input, InputEvent, Screen};
use sdl2;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

pub struct Emulator {
    cpu: CPU,
    mmu: MMU,
    input: Input,
    _screen: Screen,
    is_paused: bool,
}

impl Emulator {
    pub fn new() -> Result<Self, String> {
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
        let now = SystemTime::now();
        let mut last = now.elapsed().unwrap();
        let framerate = Duration::from_micros(16_670);
        let mut opcount: u128 = 0;

        'program: loop {
            let current = now.elapsed().unwrap();

            // Handle program I/O (events that affect the emulator). This needs to be
            match self.input.get_event() {
                InputEvent::Exit => break 'program,
                InputEvent::ToggleRun => self.is_paused = !self.is_paused,
                _ => (),
            }

            if current - last >= framerate {
                last = current;

                'frame: loop {
                    opcount += self.cpu.step(&mut self.mmu) as u128;

                    if opcount >= (4194304 / 4 / 60) {
                        opcount = 0;
                        break 'frame;
                    }
                }
            }

            // Regulate the hot loop a little. Calibrate later.
            // Sleep for at least 10 milliseconds. This needs to be replaced with something better.
            // The idea is not to run the program loop too hot for no reason, but we want to be
            // fast enough not to miss a frame and not to make I/O feel clunky.  But too fast
            // and we're spinning unnecessarily.
            sleep(Duration::from_millis(10));
        }
    }
}
