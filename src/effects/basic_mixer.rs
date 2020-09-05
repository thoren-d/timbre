use crate::{
    core::{AudioBuffer, AudioFormat, AudioSource, SharedAudioSource},
    ReadResult,
};

use slotmap::{DefaultKey, DenseSlotMap};

use tracing::trace_span;

pub struct BasicMixer {
    buffer: Vec<f32>,
    coefficient: Option<f32>,
    format: AudioFormat,
    sources: DenseSlotMap<DefaultKey, SharedAudioSource>,
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

    pub fn add_source(&mut self, source: SharedAudioSource) -> BasicMixerSource {
        BasicMixerSource {
            key: self.sources.insert(source),
        }
    }

    pub fn remove_source(&mut self, source: BasicMixerSource) {
        self.sources.remove(source.key);
    }
}

impl AudioSource for BasicMixer {
    fn read(&mut self, buffer: &mut AudioBuffer) -> ReadResult {
        let span = trace_span!("BasicMixer::read");
        let _span = span.enter();

        if self.sources.is_empty() {
            let samples = &mut buffer.samples;
            samples.iter_mut().for_each(|sample| *sample = 0.0);
            return ReadResult::good(samples.len());
        }

        let mut iter = self.sources.iter_mut();
        let (_, first) = iter.next().unwrap();
        let ReadResult {
            mut read,
            state: _state,
        } = first.lock().unwrap().read(buffer);

        for (_, source) in iter {
            let samples = &mut buffer.samples;
            self.buffer.resize(samples.len(), 0.0);

            let mut buffer = AudioBuffer::new(self.format, &mut self.buffer[..]);

            let result = source.lock().unwrap().read(&mut buffer);
            read = std::cmp::max(read, result.read);

            samples
                .iter_mut()
                .zip(self.buffer.iter())
                .for_each(|(a, b)| *a += *b);
        }

        if let Some(coef) = self.coefficient {
            buffer.samples.iter_mut().for_each(|sample| *sample *= coef);
        }

        if read < buffer.samples.len() {
            ReadResult::underrun(read)
        } else {
            ReadResult::good(buffer.samples.len())
        }
    }
}
