use sdl2::{
    self,
    audio::{AudioCallback, AudioDevice, AudioSpecDesired},
};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::time::Duration;

use crate::emulator::{AUDIO_BUFFER, AUDIO_FREQ};

// TODO: explain that this was borrowed from
// https://github.com/min050820/rust-gb/blob/master/src/gameboy/sound.rs#L137
struct Callback {
    receiver: Receiver<[[f32; 2]; AUDIO_BUFFER]>,
}

impl AudioCallback for Callback {
    type Channel = f32;
    fn callback(&mut self, buf: &mut [f32]) {
        match self.receiver.recv_timeout(Duration::from_millis(30)) {
            Ok(n) => {
                println!("{} {}", buf.len(), n.len());
                for i in 0..n.len() {
                    buf[i * 2] = n[i][0]; // Left Channel
                    buf[i * 2 + 1] = n[i][1]; // Right Channel
                }
            }
            Err(_) => {
                println!("ERROR");
                // TODO: set buffer to zeros?
            }
        }
    }
}

pub struct Audio {
    sender: SyncSender<[[f32; 2]; AUDIO_BUFFER]>,
    _player: AudioDevice<Callback>, // Not referenced but must be held or is dropped.
}

impl Audio {
    pub fn new(context: &sdl2::Sdl) -> Result<Self, String> {
        let (sender, receiver) = sync_channel(4);

        let audio = context.audio()?;
        let spec = AudioSpecDesired {
            freq: Some(AUDIO_FREQ as i32),
            channels: Some(2),
            samples: Some(AUDIO_BUFFER as u16),
        };

        let player = audio.open_playback(None, &spec, |spec| Callback { receiver })?;

        player.resume();

        Ok(Self {
            sender,
            _player: player,
        })
    }

    pub fn enqueue(&self, samples: [[f32; 2]; AUDIO_BUFFER]) {
        self.sender.send(samples).unwrap();
    }
}
