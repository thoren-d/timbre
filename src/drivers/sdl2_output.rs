use crate::{
    core::{AudioBuffer, SharedAudioSource},
    AudioFormat, StreamState,
};

use sdl2::audio::{AudioCallback, AudioFormatNum, AudioSpecDesired};
use tracing::{info, trace_span, warn};

struct Callback {
    pub format: AudioFormat,
    pub source: Option<SharedAudioSource>,
}

impl AudioCallback for Callback {
    type Channel = f32;
    fn callback(&mut self, samples: &mut [Self::Channel]) {
        let span = trace_span!("Sdl2Output::callback");
        let _span = span.enter();

        if let Some(source) = &self.source {
            let mut source = source.lock().unwrap();

            let mut buffer = AudioBuffer::new(self.format, samples);
            let result = source.read(&mut buffer);

            if result.state == StreamState::Underrun {
                warn!("Underrun detected.");
            }

            samples
                .iter_mut()
                .skip(result.read)
                .for_each(|s| *s = AudioFormatNum::SILENCE);
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
        Sdl2Output::with_format(
            subsystem,
            AudioFormat {
                channels: 2,
                sample_rate: 44100,
            },
        )
    }

    pub fn with_format(subsystem: &sdl2::AudioSubsystem, format: AudioFormat) -> Sdl2Output {
        let desired_spec = AudioSpecDesired {
            freq: Some(format.sample_rate as i32),
            channels: Some(format.channels),
            samples: Some(1024),
        };

        let device = subsystem
            .open_playback(None, &desired_spec, |spec| {
                info!("Output Spec: {:?}", spec);

                Callback {
                    format: spec.into(),
                    source: None,
                }
            })
            .unwrap();

        Sdl2Output { device }
    }

    pub fn set_source(&mut self, source: SharedAudioSource) {
        self.device.lock().source = Some(source);
    }

    pub fn format(&mut self) -> AudioFormat {
        self.device.lock().format
    }

    pub fn pause(&mut self) {
        self.device.pause();
    }

    pub fn resume(&mut self) {
        self.device.resume();
    }
}
