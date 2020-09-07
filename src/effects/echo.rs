use crate::{
    core::{AudioBuffer, AudioSource, SharedAudioSource},
    ReadResult,
};

use tracing::instrument;

/// An effect that simulates an echo.
///
/// # Examples
/// ```no_run
/// # use timbre::{generators::SineWave, effects::Echo, IntoShared};
/// # use std::time::Duration;
/// let sin = SineWave::new(1.0, 440.0);
/// let echo = Echo::new(sin.into_shared(), Duration::from_secs_f32(0.5), 0.8);
/// ```
pub struct Echo {
    source: SharedAudioSource,
    delay: f32,
    decay: f32,
    buffer: Vec<f32>,
    position: usize,
}

impl Echo {
    /// Construct a new `Echo` effect.
    ///
    /// # Arguments
    ///
    /// * `source` -- The source of audio for this effect.
    /// * `delay` -- The length of time before the echo plays back.
    /// * `decay` -- The amount by which to decay the echo on each repitition. Should
    ///              be between 0.0 and 1.0, unless you like feedback.
    pub fn new(source: SharedAudioSource, delay: std::time::Duration, decay: f32) -> Self {
        let delay = delay.as_secs_f32();
        Echo {
            source,
            delay,
            decay,
            buffer: Vec::new(),
            position: 0,
        }
    }
}

impl AudioSource for Echo {
    #[instrument(name = "Echo::read", skip(self, buffer))]
    fn read(&mut self, buffer: &mut AudioBuffer) -> ReadResult {
        let delay: usize = (buffer.format.sample_rate as f32 * self.delay).ceil() as usize
            * buffer.format.channels as usize;
        self.buffer.resize(delay, 0.0);

        let status = self.source.lock().unwrap().read(buffer);
        let written = status.read;

        echo(
            &mut self.buffer,
            buffer.samples,
            written,
            &mut self.position,
            delay,
            self.decay,
        );

        status
    }
}

fn echo(
    buffer: &mut Vec<f32>,
    samples: &mut [f32],
    written: usize,
    position: &mut usize,
    delay: usize,
    decay: f32,
) {
    let mut i = 0;
    while i < written {
        let count = std::cmp::min(delay - *position, written - i);
        (&mut buffer[*position..delay])
            .iter_mut()
            .zip((&mut samples[i..written]).iter_mut())
            .for_each(|(b, s)| {
                *b = *b * decay + *s;
                *s = *b;
            });

        i += count;
        *position = (*position + count) % delay;
    }
}
