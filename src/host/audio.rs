use sdl2::{
    self,
    audio::{AudioQueue, AudioSpecDesired},
};

use crate::emulator::{AUDIO_BUFFER, AUDIO_FREQ, CPU_FREQ};

pub struct Audio {
    player: AudioQueue<f32>,
}

impl Audio {
    pub fn new(context: &sdl2::Sdl) -> Result<Self, String> {
        let audio = context.audio()?;
        let spec = AudioSpecDesired {
            freq: Some(AUDIO_FREQ as i32),
            channels: Some(2),
            samples: Some((AUDIO_BUFFER * 4) as u16),
        };

        let player = audio.open_queue::<f32, _>(None, &spec)?;
        player.resume();

        Ok(Self { player })
    }

    pub fn enqueue(&self, sample: [f32; 2]) {
        self.player.queue(&sample);
    }
}

// TODO: might not need this.

// The number of CPU (at 4MHz) cycles that pass for each audio output sample.
// Making this a slight bit lower means we issue samples slightly more often.
// This should result in the audio buffer very slowly falling out of sync as it grows.
// But if we don't do this, there's gaps in audio, even if we queue up a bunch of quiet ahead of time.
const CYCLES_PER_SAMPLE: usize = (CPU_FREQ / AUDIO_FREQ) - 1;
