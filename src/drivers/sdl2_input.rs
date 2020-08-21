use crate::{AudioFormat, AudioSource, StreamState};

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use tracing::trace_span;

pub struct Sdl2Input {
    device: sdl2::audio::AudioDevice<Callback>,
    buffer: Arc<Mutex<VecDeque<f32>>>,
}

struct Callback {
    pub format: AudioFormat,
    pub buffer: Arc<Mutex<VecDeque<f32>>>,
}

struct AudioSourceImpl {
    pub format: AudioFormat,
    pub buffer: Arc<Mutex<VecDeque<f32>>>,
}

impl AudioCallback for Callback {
    type Channel = f32;
    fn callback(&mut self, samples: &mut [Self::Channel]) {
        let span = trace_span!("Sdl2Input::callback");
        let _span = span.enter();

        self.buffer.lock().unwrap().extend(samples.iter().cloned());
    }
}

impl Sdl2Input {
    pub fn new(subsystem: &sdl2::AudioSubsystem) -> Self {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(2),
            samples: None,
        };

        let buffer = Arc::new(Mutex::new(VecDeque::new()));

        let device = subsystem
            .open_capture(None, &desired_spec, |spec| {
                println!("Input Spec: {:?}", spec);

                Callback {
                    buffer: buffer.clone(),
                    format: spec.into(),
                }
            })
            .unwrap();

        Sdl2Input { device, buffer }
    }

    pub fn source(&mut self) -> Arc<Mutex<dyn AudioSource + Send>> {
        Arc::new(Mutex::new(AudioSourceImpl {
            buffer: self.buffer.clone(),
            format: self.device.lock().format,
        }))
    }

    pub fn resume(&mut self) {
        self.device.resume();
    }

    pub fn pause(&mut self) {
        self.device.pause();
    }
}

impl AudioSource for AudioSourceImpl {
    fn format(&mut self) -> AudioFormat {
        self.format
    }

    fn read(&mut self, samples: &mut [f32]) -> StreamState {
        let span = trace_span!("Sdl2Input::read");
        let _span = span.enter();

        let mut buffer = self.buffer.lock().unwrap();

        let mut i: usize = 0;
        while i < samples.len() {
            if let Some(sample) = buffer.pop_front() {
                samples[i] = sample;
            } else {
                return StreamState::Underrun(i);
            }
            i += 1;
        }

        StreamState::Good
    }
}
