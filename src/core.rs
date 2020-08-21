use std::sync::{Arc, Mutex};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AudioFormat {
    Mono(i32),
    Stereo(i32),
}

#[derive(Debug)]
pub enum StreamState {
    Good,
    Underrun(usize),
    Finished(usize),
}

pub trait AudioSource {
    fn format(&mut self) -> AudioFormat;
    fn read(&mut self, samples: &mut [f32]) -> StreamState;
}

pub trait Share {
    fn share(self) -> Arc<Mutex<Self>>;
}

impl<T: AudioSource + Send> Share for T {
    fn share(self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(self))
    }
}
