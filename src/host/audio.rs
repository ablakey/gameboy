use sdl2::{
    self,
    audio::{AudioCallback, AudioSpecDesired},
};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::time::Duration;

const BUFFER_SIZE: usize = 4;

// TODO: explain that this was borrowed from
// https://github.com/min050820/rust-gb/blob/master/src/gameboy/sound.rs#L137
struct Callback {
    receiver: Receiver<[[f32; 2]; 256]>,
}

impl AudioCallback for Callback {
    type Channel = f32;
    fn callback(&mut self, buf: &mut [f32]) {
        // Drain the next 256 samples.
        match self.receiver.recv_timeout(Duration::from_millis(30)) {
            Ok(n) => {
                for i in 0..n.len() {
                    buf[i * 2] = n[i][0]; // Left Channel
                    buf[i * 2 + 1] = n[i][1]; // Right Channel
                }
            }
            Err(_) => {
                // TODO: set buffer to zeros?
            }
        }
    }
}

pub struct Audio {
    sender: SyncSender<[[f32; 2]; 256]>,
}

impl Audio {
    pub fn new(context: &sdl2::Sdl) -> Result<Self, String> {
        let (sender, receiver) = sync_channel(BUFFER_SIZE);

        let audio = context.audio()?;
        let spec = AudioSpecDesired {
            freq: Some(48_000),
            channels: Some(2),
            samples: None, // Default.
        };

        let player = audio.open_playback(None, &spec, |spec| {
            println!("Open audio device: {:?}", spec);
            Callback { receiver }
        })?;

        player.resume();

        Ok(Self { sender })
    }

    pub fn enqueue(&self, samples: [[f32; 2]; 256]) {
        self.sender.send(samples);
    }
}
