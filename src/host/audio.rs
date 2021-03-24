use sdl2::{
    self,
    audio::{AudioQueue, AudioSpecDesired},
};

use crate::emulator::{AUDIO_BUFFER, AUDIO_FREQ};

pub struct Audio {
    player: AudioQueue<f32>,
}

impl Audio {
    pub fn new(context: &sdl2::Sdl) -> Result<Self, String> {
        let audio = context.audio()?;
        let spec = AudioSpecDesired {
            freq: Some(AUDIO_FREQ as i32),
            channels: Some(2),
            samples: Some(AUDIO_BUFFER as u16),
        };

        let player = audio.open_queue::<f32, _>(None, &spec)?;
        player.resume();

        Ok(Self { player })
    }

    pub fn enqueue(&self, sample: [f32; 2]) {
        self.player.queue(&sample);

        // TODO: A better approach to "catching up".
        if self.player.size() > 20_000 {
            self.player.clear();
        }

        println!("{}", self.player.size());
    }
}
