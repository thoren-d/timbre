use crate::{
    core::{AudioSource, SharedAudioSource},
    ReadResult, Sample,
};

use slotmap::{DefaultKey, DenseSlotMap};

use tracing::instrument;

/// A mixer that combines multiple [`AudioSource`](crate::AudioSource)s.
///
/// This simple mixer adds the samples from the given sources together,
/// and optionally multiplies by a coefficient to give some headroom.
///
/// # Examples
/// ```
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
        assert!(self.sources.is_empty() || source.format() == self.format());
        BasicMixerSource {
            key: self.sources.insert(source),
        }
    }

    /// Removes the source indicated by `source`, if present.
    ///
    /// # Examples
    /// ```
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
    fn format(&self) -> crate::AudioFormat {
        self.sources.iter().next().unwrap().1.format()
    }

    #[instrument(name = "BasicMixer::read", skip(self, buffer))]
    fn read(&mut self, buffer: &mut [Sample]) -> ReadResult {
        if self.sources.is_empty() {
            buffer.iter_mut().for_each(|sample| *sample = 0.0);
            return ReadResult::good(buffer.len());
        }

        let mut iter = self.sources.iter_mut();
        let (_, first) = iter.next().unwrap();
        let ReadResult {
            mut read,
            state: _state,
        } = first.lock().unwrap().read(buffer);

        for (_, source) in iter {
            self.buffer.resize(buffer.len(), 0.0);

            let result = source.lock().unwrap().read(&mut self.buffer);
            read = std::cmp::max(read, result.read);

            buffer
                .iter_mut()
                .zip(self.buffer.iter())
                .for_each(|(a, b)| *a += *b);
        }

        if let Some(coef) = self.coefficient {
            buffer.iter_mut().for_each(|sample| *sample *= coef);
        }

        if read < buffer.len() {
            ReadResult::underrun(read)
        } else {
            ReadResult::good(buffer.len())
        }
    }
}
