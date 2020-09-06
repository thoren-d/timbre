use crate::{
    core::{AudioBuffer, AudioSource, SharedAudioSource},
    ReadResult,
};

use slotmap::{DefaultKey, DenseSlotMap};

use tracing::trace_span;

/// A mixer that combines multiple [`AudioSource`](crate::AudioSource)s.
///
/// This simple mixer adds the samples from the given sources together,
/// and optionally multiplies by a coefficient to give some headroom.
///
/// # Examples
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use timbre::{generators::SineWave, effects::BasicMixer, IntoShared};
/// let sin1 = SineWave::new(0.5, 440.0);
/// let sin2 = SineWave::new(0.5, 220.0);
///
/// let mut mixer = BasicMixer::new();
/// let sin1 = mixer.add_source(sin1.into_shared());
/// mixer.add_source(sin2.into_shared());
/// mixer.remove_source(sin1);
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct BasicMixer {
    buffer: Vec<f32>,
    coefficient: Option<f32>,
    sources: DenseSlotMap<DefaultKey, SharedAudioSource>,
}

/// A key used to remove sources that have been added to [`BasicMixer`](crate::effects::BasicMixer).
pub struct BasicMixerSource {
    key: DefaultKey,
}

impl BasicMixer {
    /// Construct a `BasicMixer` that simply adds samples and doesn't multiply by anything.
    pub fn new() -> Self {
        BasicMixer {
            coefficient: None,
            sources: DenseSlotMap::new(),
            buffer: Vec::new(),
        }
    }

    /// Construct a `BasicMixer` that adds samples together, then multiplies by the given
    /// coefficient to reduce the chance of clipping.
    ///
    /// # Arguments
    ///
    /// * `coefficient` -- A number to multiply the final resulting samples by.
    pub fn with_coefficient(coefficient: f32) -> Self {
        BasicMixer {
            buffer: Vec::new(),
            coefficient: Some(coefficient),
            sources: DenseSlotMap::new(),
        }
    }

    /// Add a source to this mixer.
    ///
    /// # Arguments
    ///
    /// * `source` -- The audio source to add to this mixer.
    ///
    /// # Returns
    ///
    /// A key to be used in [`remove_source`](method.remove_source) to remove this source.
    pub fn add_source(&mut self, source: SharedAudioSource) -> BasicMixerSource {
        BasicMixerSource {
            key: self.sources.insert(source),
        }
    }

    /// Removes the source indicated by `source`, if present.
    ///
    /// # Examples
    /// ```no_run
    /// # use timbre::{effects::BasicMixer, generators::SineWave, IntoShared};
    /// let sin = SineWave::new(1.0, 440.0);
    /// let mut mixer = BasicMixer::new();
    /// let sin = mixer.add_source(sin.into_shared());
    /// mixer.remove_source(sin);
    /// ```
    pub fn remove_source(&mut self, source: BasicMixerSource) {
        self.sources.remove(source.key);
    }
}

impl AudioSource for BasicMixer {
    fn read(&mut self, buffer: &mut AudioBuffer) -> ReadResult {
        let span = trace_span!("BasicMixer::read");
        let _span = span.enter();

        if self.sources.is_empty() {
            buffer.samples.iter_mut().for_each(|sample| *sample = 0.0);
            return ReadResult::good(buffer.samples.len());
        }

        let mut iter = self.sources.iter_mut();
        let (_, first) = iter.next().unwrap();
        let ReadResult {
            mut read,
            state: _state,
        } = first.lock().unwrap().read(buffer);

        for (_, source) in iter {
            self.buffer.resize(buffer.samples.len(), 0.0);

            {
                let mut buffer = AudioBuffer {
                    format: buffer.format,
                    samples: &mut self.buffer[..],
                };

                let result = source.lock().unwrap().read(&mut buffer);
                read = std::cmp::max(read, result.read);
            }

            buffer
                .samples
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
