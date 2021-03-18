use sdl2::{
    self,
    audio::{AudioCallback, AudioQueue, AudioSpecDesired},
};
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::emulator::{AUDIO_BUFFER, AUDIO_FREQ};

// TODO: explain that this was borrowed from
// https://github.com/min050820/rust-gb/blob/master/src/gameboy/sound.rs#L137
struct Callback {
    receiver: Receiver<[[f32; 2]; AUDIO_BUFFER]>,
}

impl AudioCallback for Callback {
    type Channel = f32;
    fn callback(&mut self, buf: &mut [f32]) {
        match self.receiver.recv() {
            Ok(n) => {
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
    sender: Sender<[[f32; 2]; AUDIO_BUFFER]>,
    _player: AudioQueue<f32>, // Not referenced but must be held or is dropped.
}

impl Audio {
    pub fn new(context: &sdl2::Sdl) -> Result<Self, String> {
        let (sender, receiver) = channel();

        let audio = context.audio()?;
        let spec = AudioSpecDesired {
            freq: Some(AUDIO_FREQ as i32),
            channels: Some(2),
            samples: None, // Default.
        };

        // let player = audio.open_playback(None, &spec, |spec| Callback { receiver })?;
        let player = audio.open_queue::<f32, _>(None, &spec)?;

        player.resume();

        Ok(Self {
            sender,
            _player: player,
        })
    }

    pub fn enqueue(&self, samples: [f32; AUDIO_BUFFER]) {
        // self._player.queue(&samples);
        self._player.queue(&gen_wave(44_100));
        // self.sender.send(samples).unwrap();
    }
}

fn gen_wave(bytes_to_write: i32) -> Vec<f32> {
    // Generate a square wave
    let tone_volume = 1_000f32;
    let period = 220;
    let sample_count = bytes_to_write;
    let mut result = Vec::new();

    for x in 0..sample_count {
        result.push(if (x / period) % 2 == 0 {
            tone_volume
        } else {
            -tone_volume
        });
    }
    result
}
