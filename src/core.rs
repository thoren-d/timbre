use std::sync::{Arc, Mutex};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct AudioFormat {
    pub channels: u8,
    pub sample_rate: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum StreamState {
    Good,
    Underrun,
    Finished,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ReadResult {
    pub state: StreamState,
    pub read: usize,
}

impl ReadResult {
    pub fn good(read: usize) -> Self {
        ReadResult {
            state: StreamState::Good,
            read,
        }
    }

    pub fn underrun(read: usize) -> Self {
        ReadResult {
            state: StreamState::Underrun,
            read,
        }
    }

    pub fn finished(read: usize) -> Self {
        ReadResult {
            state: StreamState::Finished,
            read,
        }
    }
}

pub type Sample = f32;

pub struct AudioBuffer<'a> {
    pub samples: &'a mut [Sample],
    pub format: AudioFormat,
}

impl<'a> AudioBuffer<'a> {
    pub fn new(format: AudioFormat, samples: &'a mut [Sample]) -> Self {
        AudioBuffer { format, samples }
    }
}

pub trait AudioSource {
    fn read(&mut self, buffer: &mut AudioBuffer) -> ReadResult;
}

pub type SharedAudioSource = Arc<Mutex<dyn AudioSource + Send>>;

pub trait IntoShared {
    fn into_shared(self) -> SharedAudioSource;
}

impl<T: AudioSource + Send + 'static> IntoShared for T {
    fn into_shared(self) -> SharedAudioSource {
        Arc::new(Mutex::new(self))
    }
}
