use crate::{AudioSource, ReadResult};
use tracing::trace_span;

pub struct SinWave {
    amplitude: f32,
    phase: f32,
    frequency: f32,
}

impl SinWave {
    pub fn new(amplitude: f32, frequency: f32) -> Self {
        SinWave {
            amplitude,
            phase: 0.0,
            frequency,
        }
    }
}

impl AudioSource for SinWave {
    fn read(&mut self, buffer: &mut crate::core::AudioBuffer) -> crate::ReadResult {
        let span = trace_span!("SinWav::read");
        let _span = span.enter();

        let increment =
            std::f32::consts::PI * 2.0 * self.frequency / buffer.format.sample_rate as f32;

        let channels = buffer.format.channels as usize;
        let frames = buffer.samples.len() / channels;

        for i in 0..frames {
            let amplitude = self.amplitude * self.phase.sin();
            for channel in 0..channels as usize {
                buffer.samples[i * channels + channel] = amplitude;
            }
            self.phase += increment;
        }

        ReadResult::good(buffer.samples.len())
    }
}
