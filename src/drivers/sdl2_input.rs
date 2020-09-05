use crate::{
    core::{AudioBuffer, SharedAudioSource},
    AudioFormat, AudioSource, ReadResult,
};

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use tracing::{info, trace_span};

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
        Sdl2Input::with_format(
            subsystem,
            AudioFormat {
                channels: 2,
                sample_rate: 44100,
            },
        )
    }

    pub fn with_format(subsystem: &sdl2::AudioSubsystem, format: AudioFormat) -> Self {
        let desired_spec = AudioSpecDesired {
            freq: Some(format.sample_rate as i32),
            channels: Some(format.channels),
            samples: Some(1024),
        };

        let buffer = Arc::new(Mutex::new(VecDeque::new()));

        let device = subsystem
            .open_capture(None, &desired_spec, |spec| {
                info!("Input Spec: {:?}", spec);

                Callback {
                    buffer: buffer.clone(),
                    format: spec.into(),
                }
            })
            .unwrap();

        Sdl2Input { device, buffer }
    }

    pub fn format(&mut self) -> AudioFormat {
        self.device.lock().format
    }

    pub fn source(&mut self) -> SharedAudioSource {
        Arc::new(Mutex::new(AudioSourceImpl {
            buffer: Arc::clone(&self.buffer),
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
    fn read(&mut self, buffer: &mut AudioBuffer) -> ReadResult {
        assert!(self.format == buffer.format);
        let samples = &mut buffer.samples;
        let span = trace_span!("Sdl2Input::read");
        let _span = span.enter();

        let mut buffer = self.buffer.lock().unwrap();

        let mut i: usize = 0;
        while i < samples.len() {
            if let Some(sample) = buffer.pop_front() {
                samples[i] = sample;
            } else {
                return ReadResult::underrun(i);
            }
            i += 1;
        }

        ReadResult::good(samples.len())
    }
}
