mod cpu;
mod input;
mod mmu;
mod opcode;
mod registers;
mod screen;

use cpu::CPU;
use input::{Input, InputEvent};
use mmu::MMU;
use screen::Screen;
use sdl2;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let emulator = Emulator::new();

    match emulator {
        Ok(mut e) => e.run_forever(),
        Err(e) => panic!("Could not launch emulator. {}", e),
    }

    // loop {
    //     emulator.step();
    // }
}

struct Emulator {
    cpu: CPU,
    input: Input,
    screen: Screen,
    is_paused: bool,
}

impl Emulator {
    fn new() -> Result<Self, String> {
        // SDL-based host: graphics, sound, audio.
        let sdl_context = sdl2::init()?;
        let input = Input::new(&sdl_context)?;
        let screen = Screen::new(&sdl_context, 4)?;
        Ok(Self {
            cpu: CPU::new(),
            input,
            is_paused: false,
            screen,
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
        self.cpu.step();
    }
}
