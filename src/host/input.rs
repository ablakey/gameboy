use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::EventPump;

#[derive(PartialEq)]
pub enum InputEvent {
    None,
    Exit,
    Panic,
}

pub struct Input {
    event_pump: EventPump,
}

const KEY_BINDINGS: [Scancode; 8] = [
    Scancode::Right, // Right
    Scancode::Left,  // Left
    Scancode::Up,    // Up
    Scancode::Down,  // Down
    Scancode::A,     // A
    Scancode::S,     // B
    Scancode::X,     // Select
    Scancode::Z,     // Start
];

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
                } => InputEvent::Panic,
                Event::KeyDown { .. } => InputEvent::None,
                _ => InputEvent::None,
            };

            if x != InputEvent::None {
                break;
            }
        }

        return x;
    }

    /// Return an array of key states. true = pressed.
    pub fn get_gamepad_state(&self) -> [bool; 8] {
        let keys: Vec<Scancode> = self
            .event_pump
            .keyboard_state()
            .pressed_scancodes()
            .collect();

        // Hard coded binding of keyboard to keys.  We use the left 16 keys in the same grid pattern
        // which means none of the letters/numbers align, but the shape does.
        let key_states = KEY_BINDINGS
            .iter()
            .map(|b| keys.contains(b))
            .collect::<Vec<bool>>();

        let mut array = [false; 8];
        array.copy_from_slice(&key_states[..]);
        array
    }
}
