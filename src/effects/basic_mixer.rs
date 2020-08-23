use crate::{
    core::{AudioFormat, AudioSource},
    ReadResult,
};

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
        assert!(source.lock().unwrap().request_format(None) == self.format);
        BasicMixerSource {
            key: self.sources.insert(source),
        }
    }

    pub fn remove_source(&mut self, source: BasicMixerSource) {
        self.sources.remove(source.key);
    }
}

impl AudioSource for BasicMixer {
    fn request_format(&mut self, _format: Option<AudioFormat>) -> AudioFormat {
        self.format
    }

    fn read(&mut self, samples: &mut [f32]) -> ReadResult {
        let span = trace_span!("BasicMixer::read");
        let _span = span.enter();

        if self.sources.is_empty() {
            samples.iter_mut().for_each(|sample| *sample = 0.0);
            return ReadResult::good(samples.len());
        }

        let mut iter = self.sources.iter_mut();
        let (_, first) = iter.next().unwrap();
        let ReadResult {
            mut read,
            state: _state,
        } = first.lock().unwrap().read(samples);

        for (_, source) in iter {
            self.buffer.resize(samples.len(), 0.0);

            let result = source.lock().unwrap().read(&mut self.buffer[..]);
            read = std::cmp::max(read, result.read);

            samples
                .iter_mut()
                .zip(self.buffer.iter())
                .for_each(|(a, b)| *a += *b);
        }

        if let Some(coef) = self.coefficient {
            samples.iter_mut().for_each(|sample| *sample *= coef);
        }

        if read < samples.len() {
            ReadResult::underrun(read)
        } else {
            ReadResult::good(samples.len())
        }
    }
}
