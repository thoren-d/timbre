use crate::core::{AudioFormat, AudioSource, StreamState};

use slotmap::{DefaultKey, DenseSlotMap};

use std::sync::{Arc, Mutex};
use tracing::trace_span;

pub struct BasicMixer {
    buffer: Vec<f32>,
    coefficient: Option<f32>,
    format: AudioFormat,
    sources: DenseSlotMap<DefaultKey, Arc<Mutex<dyn AudioSource + Send>>>,
}

pub struct BasicMixerSource {
    key: DefaultKey,
}

impl BasicMixer {
    pub fn new(format: AudioFormat, coefficient: Option<f32>) -> Self {
        BasicMixer {
            coefficient,
            format,
            sources: DenseSlotMap::new(),
            buffer: Vec::new(),
        }
    }

    pub fn add_source(&mut self, source: Arc<Mutex<dyn AudioSource + Send>>) -> BasicMixerSource {
        assert!(source.lock().unwrap().format() == self.format);
        BasicMixerSource {
            key: self.sources.insert(source),
        }
    }

    pub fn remove_source(&mut self, source: BasicMixerSource) {
        self.sources.remove(source.key);
    }
}

impl AudioSource for BasicMixer {
    fn format(&mut self) -> AudioFormat {
        self.format
    }

    fn read(&mut self, samples: &mut [f32]) -> StreamState {
        let span = trace_span!("BasicMixer::read");
        let _span = span.enter();

        if self.sources.is_empty() {
            samples.iter_mut().for_each(|sample| *sample = 0.0);
            return StreamState::Good;
        }

        let mut iter = self.sources.iter_mut();
        let (_, first) = iter.next().unwrap();
        let mut written = match first.lock().unwrap().read(samples) {
            StreamState::Good => samples.len(),
            StreamState::Finished(n) => n,
            StreamState::Underrun(n) => n,
        };

        for (_, source) in iter {
            self.buffer.resize(samples.len(), 0.0);

            written = std::cmp::max(
                written,
                match source.lock().unwrap().read(&mut self.buffer[..]) {
                    StreamState::Good => samples.len(),
                    StreamState::Finished(n) => n,
                    StreamState::Underrun(n) => n,
                },
            );

            samples
                .iter_mut()
                .zip(self.buffer.iter())
                .for_each(|(a, b)| *a += *b);
        }

        if let Some(coef) = self.coefficient {
            samples.iter_mut().for_each(|sample| *sample *= coef);
        }

        if written < samples.len() {
            StreamState::Underrun(written)
        } else {
            StreamState::Good
        }
    }
}
