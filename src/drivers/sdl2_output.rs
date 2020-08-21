use crate::{AudioFormat, AudioSource, StreamState};

use std::sync::{Arc, Mutex};

use sdl2::audio::{AudioCallback, AudioFormatNum, AudioSpecDesired};
use tracing::trace_span;

struct Callback {
    pub format: AudioFormat,
    pub source: Option<Arc<Mutex<dyn AudioSource + Send>>>,
}

impl AudioCallback for Callback {
    type Channel = f32;
    fn callback(&mut self, samples: &mut [Self::Channel]) {
        let span = trace_span!("Sdl2Output::callback");
        let _span = span.enter();

        if let Some(source) = &self.source {
            let mut source = source.lock().unwrap();
            if source.format() != self.format {
                panic!("Incompatible source format.");
            }

            let result = source.read(samples);

            match result {
                StreamState::Good => {}
                StreamState::Underrun(n) => {
                    println!("Underrun detected.");
                    samples
                        .iter_mut()
                        .skip(n)
                        .for_each(|sample| *sample = AudioFormatNum::SILENCE);
                }
                StreamState::Finished(n) => {
                    samples
                        .iter_mut()
                        .skip(n)
                        .for_each(|sample| *sample = AudioFormatNum::SILENCE);
                }
            }
        } else {
            for sample in samples.iter_mut() {
                *sample = AudioFormatNum::SILENCE;
            }
        }
    }
}

pub struct Sdl2Output {
    device: sdl2::audio::AudioDevice<Callback>,
}

impl Sdl2Output {
    pub fn new(subsystem: &sdl2::AudioSubsystem) -> Sdl2Output {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(2),
            samples: None,
        };

        let device = subsystem
            .open_playback(None, &desired_spec, |spec| {
                println!("Output Spec: {:?}", spec);

                Callback {
                    format: spec.into(),
                    source: None,
                }
            })
            .unwrap();

        Sdl2Output { device }
    }

    pub fn set_source(&mut self, source: Arc<Mutex<dyn AudioSource + Send>>) {
        self.device.lock().source = Some(source);
    }

    pub fn pause(&mut self) {
        self.device.pause();
    }

    pub fn resume(&mut self) {
        self.device.resume();
    }
}
