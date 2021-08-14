use crate::{core::AudioSource, ReadResult, Sample};

use tracing::instrument;

/// An effect that suppresses low frequencies.
///
/// `HighPass` reduces the volume of frequencies below the given cutoff.
/// This can create the impression of sound played on old speakers or a radio.
///
/// # Examples
/// ```
/// # use timbre::{generators::SineWave, effects::HighPass, IntoShared};
/// # use std::time::Duration;
/// let sin = SineWave::new(1.0, 440.0);
/// let high_pass = HighPass::new(sin, 4000.0);
/// ```
pub struct HighPass<S: AudioSource> {
    buffer: Vec<f32>,
    rc: f32,
    source: S,
    prev: [f32; 2],
}

impl<S: AudioSource> HighPass<S> {
    /// Construct a high-pass filter.
    ///
    /// # Arguments
    ///
    /// * `source` -- The source of audio for this effect.
    /// * `cutoff` -- The frequency below which volume will be reduced.
    pub fn new(source: S, cutoff: f32) -> Self {
        let buffer = Vec::new();
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
        HighPass {
            buffer,
            rc,
            source,
            prev: [0.0, 0.0],
        }
    }
}

impl<S: AudioSource> AudioSource for HighPass<S> {
    fn format(&self) -> crate::AudioFormat {
        self.source.format()
    }

    #[instrument(name = "HighPass::read", skip(self, buffer))]
    fn read(&mut self, buffer: &mut [Sample]) -> ReadResult {
        let format = self.source.format();
        let result = self.source.read(buffer);
        let written = result.read;
        if written == 0 {
            return result;
        }
        self.buffer.resize(buffer.len(), 0.0);

        match format.channels {
            1 => {
                let dt = 1.0 / format.sample_rate as f32;
                self.prev = filter_mono(
                    &mut buffer[..written],
                    &mut self.buffer[..written],
                    dt,
                    self.rc,
                    self.prev,
                );
            }
            2 => {
                let dt = 1.0 / format.sample_rate as f32;
                self.prev = filter_stereo(
                    &mut buffer[..written],
                    &mut self.buffer[..written],
                    dt,
                    self.rc,
                    self.prev,
                );
            }
            _ => panic!("Unsupported channel count."),
        }

        result
    }
}

fn filter_mono(
    samples: &mut [f32],
    buffer: &mut [f32],
    dt: f32,
    rc: f32,
    prev: [f32; 2],
) -> [f32; 2] {
    assert!(!samples.is_empty() && !buffer.is_empty());
    assert!(buffer.len() >= samples.len());

    let a = rc / (rc + dt);
    let res = [samples[samples.len() - 1], 0.0];

    buffer[0] = a * (buffer[buffer.len() - 1] + samples[0] - prev[0]);
    for i in 1..buffer.len() {
        buffer[i] = a * (buffer[i - 1] + samples[i] - samples[i - 1]);
    }
    samples.copy_from_slice(&buffer[..samples.len()]);

    res
}

fn filter_stereo(
    samples: &mut [f32],
    buffer: &mut [f32],
    dt: f32,
    rc: f32,
    prev: [f32; 2],
) -> [f32; 2] {
    assert!(!samples.is_empty() && !buffer.is_empty());
    assert!(samples.len() % 2 == 0 && buffer.len() % 2 == 0);
    assert!(buffer.len() >= samples.len());

    let a = rc / (rc + dt);
    let res = [samples[samples.len() - 2], samples[samples.len() - 1]];

    buffer[0] = a * (buffer[buffer.len() - 2] + samples[0] - prev[0]);
    buffer[1] = a * (buffer[buffer.len() - 1] + samples[1] - prev[1]);
    for i in 2..buffer.len() {
        buffer[i] = a * (buffer[i - 2] + samples[i] - samples[i - 2]);
    }
    samples.copy_from_slice(&buffer[..samples.len()]);

    res
}
