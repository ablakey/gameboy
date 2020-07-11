use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::EventPump;

#[derive(PartialEq)]
pub enum InputEvent {
    None,
    Exit,
    ToggleRun,
    Tick,
    SaveState,
    RestoreState,
}

pub struct Input {
    event_pump: EventPump,
}

impl Input {
    pub fn new(context: &sdl2::Sdl) -> Result<Self, String> {
        let event_pump = context.event_pump()?;

        Ok(Self { event_pump })
    }

    /// Return a single, highest priority event.
    /// This may be a call to quit the application, change a debug setting, or supply keyboard
    /// state to the emulator.
    pub fn get_event(&mut self) -> InputEvent {
        let mut x = InputEvent::None;

        for event in self.event_pump.poll_iter() {
            x = match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => InputEvent::Exit,
                Event::KeyUp {
                    keycode: Some(Keycode::Space),
                    ..
                } => InputEvent::ToggleRun,
                Event::KeyUp {
                    keycode: Some(Keycode::F5),
                    ..
                } => InputEvent::SaveState,
                Event::KeyUp {
                    keycode: Some(Keycode::F9),
                    ..
                } => InputEvent::RestoreState,
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => InputEvent::Tick,
                Event::KeyDown { .. } => InputEvent::None,
                _ => InputEvent::None,
            };

            if x != InputEvent::None {
                break;
            }
        }

        return x;
    }
}
