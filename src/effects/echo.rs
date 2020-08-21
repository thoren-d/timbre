use crate::core::{AudioFormat, AudioSource, StreamState};

use std::sync::{Arc, Mutex};
use tracing::trace_span;

pub struct Echo {
    source: Arc<Mutex<dyn AudioSource + Send>>,
    delay: usize,
    decay: f32,
    buffer: Vec<f32>,
    position: usize,
}

impl Echo {
    pub fn new(source: Arc<Mutex<dyn AudioSource + Send>>, delay: usize, decay: f32) -> Self {
        let mut buffer = Vec::new();
        buffer.resize(delay, 0.0);
        Echo {
            source,
            delay,
            decay,
            buffer,
            position: 0,
        }
    }
}

impl AudioSource for Echo {
    fn format(&mut self) -> AudioFormat {
        self.source.lock().unwrap().format()
    }

    fn read(&mut self, samples: &mut [f32]) -> StreamState {
        let span = trace_span!("Echo::read");
        let _span = span.enter();

        let status = self.source.lock().unwrap().read(samples);
        let written = match status {
            StreamState::Good => samples.len(),
            StreamState::Finished(n) => n,
            StreamState::Underrun(n) => n,
        };

        echo(
            &mut self.buffer,
            samples,
            written,
            &mut self.position,
            self.delay,
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
