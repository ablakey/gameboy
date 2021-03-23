use crate::guest::systems::{Gamepad, Timer, APU, CPU, PPU};
use crate::guest::MMU;
use crate::host::{Audio, Input, InputEvent, Screen};
use sdl2;
use std::time::Duration;
use tokio;

pub const CPU_FREQ: usize = 4194304; // 4MHz for DMG-01.
pub const AUDIO_FREQ: usize = 48_000; // 48KHz audio sample target.
pub const AUDIO_BUFFER: usize = 1024; // Needs to be a power of 2 and more than 1 frame of sound.
pub const DIVIDER_FREQ: usize = CPU_FREQ / 16384; // Divider always runs at 16KHz.

// Emulate audio a fraction as often as the actual frequency.
// If a single CPU instruction occurs, it is a minimum of 4 CPU clock cycles. We could emulate 4 APU
// steps, but that provides such a crazy high number of sound samples that we don't need. We'll run
// each voice's ticks a fraction as often, but still count all cycles (ie. a single tick is treated
// APU_DIVISOR number of cycles)
pub const APU_DIVISOR: usize = 4;

// APU generates samples at some frequency that's far higher than the audio device.
// This is how many APU samples should be used to generate a single audio device sample.
const APU_SAMPLES_PER_AUDIO_SAMPLE: usize = (CPU_FREQ / APU_DIVISOR) / AUDIO_FREQ;

const FRAMERATE: usize = 60;

pub struct Emulator {
    // Guest components.
    cpu: CPU,
    ppu: PPU,
    mmu: MMU,
    apu: APU,
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
            apu: APU::new(),
            timer: Timer::new(),
            gamepad: Gamepad::new(),
            input,
            audio,
            screen,
        })
    }

    pub async fn run_forever(&mut self) {
        'program: loop {
            // Handle program I/O (events that affect the emulator). This needs to be
            match self.input.get_event() {
                InputEvent::Exit => break 'program,
                InputEvent::Panic => panic!("Panic caused by user."),
                _ => (),
            }
            self.emulate_frame().await;
        }
    }

    /// Emulate one whole frame work of CPU, PPU, Timer work. Given 60fps, 1 frame is 1/60 of the
    /// CPU clock speed worth of work:
    async fn emulate_frame(&mut self) {
        let mmu = &mut self.mmu;
        let mut cycle_count: usize = 0;

        // Update gamepad input state. Do this at 60hz to save on CPU.
        let gamepad_state = self.input.get_gamepad_state();
        self.gamepad.update_state(gamepad_state);

        'frame: loop {
            // Advance each emulator system one opcode (step).
            // The length of the step depends on what opcode is executed.
            self.gamepad.step(mmu);
            let cycles = self.cpu.step(mmu);
            self.timer.step(mmu, cycles);
            self.ppu.step(mmu, cycles);
            self.apu.step(mmu, cycles);

            // 4Mhz cpu at 60fps.
            cycle_count += cycles as usize;
            if cycle_count >= (CPU_FREQ / FRAMERATE) {
                break 'frame;
            }
        }

        // Drain the entire contents of the emulator's audio sample buffer into the host's buffer.
        // Recall: the host accepts a vector of any size, but it feeds that vector into an MPSC
        // that will block when full.  The audio device will drain this buffer in a separate thread.
        while self.apu.output_buffer.len() >= APU_SAMPLES_PER_AUDIO_SAMPLE {
            let x: Vec<[f32; 2]> = self
                .apu
                .output_buffer
                .drain(0..APU_SAMPLES_PER_AUDIO_SAMPLE)
                .collect();
            let y: f32 = x.iter().map(|n| n[0]).sum::<f32>() / x.len() as f32;
            self.audio.enqueue([y, y]);
            // TODO: doing a lot of probably inefficient work here, and cutting out audio channel.

            // println!("{:?}", self.apu.output_buffer);
        }

        println!("{}", self.apu.output_buffer.len());

        // Prevent CPU blocking unnecessary time in screen.update (SDL2 vsync blocks).
        // This is kind of bad because it assumes things about hardware performance and how long
        // the above functions and screen.update take.
        // Ideally, this value is "16.67ms - emulation_time - audio_time - screen_time."
        // That way we give screen.update just enough time to update and it does very little
        // vsync blocking.  There might be a way to implement that by tracking how long past frames
        // took.
        tokio::time::sleep(Duration::from_millis(5)).await;

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
