use super::guest::{CPU, GPU, MMU};
use super::host::{Input, InputEvent, Screen};
use sdl2;
use std::time::{Duration, SystemTime};

pub struct Emulator {
    cpu: CPU,
    gpu: GPU,
    mmu: MMU,
    input: Input,
    _screen: Screen,
    is_paused: bool,
    now: SystemTime,
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
            gpu: GPU::new(),
            input,
            is_paused: false,
            _screen,
            now: SystemTime::now(),
        })
    }

    pub fn run_forever(&mut self) {
        let mut last = self.now.elapsed().unwrap();

        'program: loop {
            let current = self.now.elapsed().unwrap();

            // Handle program I/O (events that affect the emulator). This needs to be
            match self.input.get_event() {
                InputEvent::Exit => break 'program,
                InputEvent::ToggleRun => self.is_paused = !self.is_paused,
                _ => (),
            }

            // If at least 16.67ms have passed since starting the last frame, process another frame.
            if current - last >= Duration::from_micros(16_670) {
                last = current;
                self.emulate_frame();
            }
        }
    }

    /// Loop at max-speed to process an entire frame.
    fn emulate_frame(&mut self) {
        let mut cycle_count: usize = 0;
        'frame: loop {
            // TODO: this loop will expand to step one line at a time through the CPU, GPU, APU.
            let cycles = self.cpu.step(&mut self.mmu);
            self.gpu.step(cycles, &mut self.mmu);
            cycle_count += cycles as usize;

            // 4Mhz cpu at 60fps.
            if cycle_count >= (4194304 / 60) {
                break 'frame;
            }
        }
    }
}
