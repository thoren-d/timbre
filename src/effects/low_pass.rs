use crate::{
    core::{AudioBuffer, AudioSource, SharedAudioSource},
    ReadResult,
};

use tracing::trace_span;

pub struct LowPass {
    buffer: Vec<f32>,
    rc: f32,
    source: SharedAudioSource,
}

impl LowPass {
    pub fn new(source: SharedAudioSource, cutoff: f32) -> Self {
        let buffer = Vec::new();
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
        LowPass { buffer, rc, source }
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
    }

    pub fn cutoff(&self) -> f32 {
        1.0 / (2.0 * std::f32::consts::PI * self.rc)
    }
}

impl AudioSource for LowPass {
    fn read(&mut self, buffer: &mut AudioBuffer) -> ReadResult {
        let span = trace_span!("LowPass::read");
        let _span = span.enter();

        let result = self.source.lock().unwrap().read(buffer);
        let written = result.read;
        if written == 0 {
            return result;
        }
        self.buffer.resize(buffer.samples.len(), 0.0);

        match buffer.format.channels {
            1 => {
                let dt = 1.0 / buffer.format.sample_rate as f32;
                filter_mono(
                    &mut buffer.samples[..written],
                    &mut self.buffer[..written],
                    dt,
                    self.rc,
                );
            }
            2 => {
                let dt = 1.0 / buffer.format.sample_rate as f32;
                filter_stereo(
                    &mut buffer.samples[..written],
                    &mut self.buffer[..written],
                    dt,
                    self.rc,
                );
            }
            _ => panic!("Unsupported channel count."),
        }

        result
    }
}

fn filter_mono(samples: &mut [f32], buffer: &mut [f32], dt: f32, rc: f32) {
    assert!(!samples.is_empty() && !buffer.is_empty());
    assert!(buffer.len() >= samples.len());

    let a = rc / (rc + dt);

    buffer[0] = buffer[buffer.len() - 1] + a * (samples[0] - buffer[buffer.len() - 1]);
    for i in 1..buffer.len() {
        buffer[i] = buffer[i - 1] + a * (samples[i] - buffer[i - 1]);
    }
    samples.copy_from_slice(&buffer[..samples.len()]);
}

fn filter_stereo(samples: &mut [f32], buffer: &mut [f32], dt: f32, rc: f32) {
    assert!(!samples.is_empty() && !buffer.is_empty());
    assert!(samples.len() % 2 == 0 && buffer.len() % 2 == 0);
    assert!(buffer.len() >= samples.len());

    let a = dt / (rc + dt);

    buffer[0] = buffer[buffer.len() - 2] + a * (samples[0] - buffer[buffer.len() - 2]);
    buffer[1] = buffer[buffer.len() - 1] + a * (samples[1] - buffer[buffer.len() - 1]);
    for i in 2..buffer.len() {
        buffer[i] = buffer[i - 2] + a * (samples[i] - buffer[i - 2]);
    }
    samples.copy_from_slice(&buffer[..samples.len()]);
}
