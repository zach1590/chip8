use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::audio::AudioDevice;
use sdl2::AudioSubsystem;
use sdl2::Sdl;
use std::ops::DerefMut;

pub struct SquareWave {
    pub phase_inc: f32,
    pub phase: f32,
    pub volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume / 100.0
            } else {
                -self.volume / 100.0
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub struct SoundSystem {
    pub audio_subsystem: AudioSubsystem,
    pub desired_spec: AudioSpecDesired,
    pub device: AudioDevice<SquareWave>,
}

impl SoundSystem{
    pub fn initialize(sdl_context: &Sdl) -> SoundSystem{
        
        let audio_subsystem = sdl_context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(41720),  //44100
            channels: Some(1),  // mono
            samples: Some(256)       
        };
        let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            SquareWave {
                phase_inc: 300.0 / spec.freq as f32,  //441
                phase: 0.0,
                volume: 0.25,//0.0015,
            }
        }).unwrap();

        return SoundSystem {
            audio_subsystem,
            desired_spec,
            device,
        }
    }

    pub fn handle_timer(self: &mut Self, sound_timer: &u8) {
        if *sound_timer > 0 {
            let mut wave_guard = self.device.lock();
            let mut wave = wave_guard.deref_mut();
            wave.volume = 0.25;
        }
        else {
            let mut wave_guard = self.device.lock();
            let mut wave = wave_guard.deref_mut();
            wave.volume = 0.0;
        }
    }
}