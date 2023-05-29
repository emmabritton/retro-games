use std::io::Cursor;
use audio_engine::{AudioEngine, Sound, WavDecoder};
use color_eyre::eyre::eyre;
use pixels_graphics_lib::Timing;
use color_eyre::Result;

pub trait NewSoundEffect {
    fn load_from_bytes(&self, bytes: &'static [u8], duration: f64) -> Result<SoundEffect>;
}

impl NewSoundEffect for AudioEngine {
    fn load_from_bytes(&self, bytes: &'static [u8], duration: f64) -> Result<SoundEffect>{
        let decoder = WavDecoder::new(Cursor::new(bytes))?;
        let sound = self.new_sound(decoder).map_err(|e| eyre!(e))?;
        Ok(SoundEffect::new(sound, duration))
    }
}

pub struct SoundEffect {
    sound: Sound,
    is_playing: bool,
    duration: f64,
    next_play_in: f64,
    loops: bool
}

impl SoundEffect {
    pub fn new(sound: Sound, duration: f64) -> Self {
        Self { sound, is_playing: false, duration, next_play_in: 0.0,loops: false }
    }

    pub fn play(&mut self) {
        if !self.is_playing {
            self.sound.play();
            self.is_playing = true;
            self.next_play_in = self.duration;
        }
    }

    pub fn reset(&mut self) {
        self.sound.stop();
        self.is_playing = false;
        self.next_play_in = 0.0;
    }

    pub fn set_loop(&mut self, loops: bool) {
        self.loops = loops;
        self.sound.set_loop(loops)
    }

    pub fn can_play(&self) -> bool {
        !self.is_playing && self.next_play_in < 0.0
    }

    pub fn update(&mut self, timing: &Timing) {
        if !self.loops && self.is_playing && self.next_play_in < 0.0 {
            self.reset();
        }
        self.next_play_in -= timing.fixed_time_step;
    }
}