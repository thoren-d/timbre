use std::sync::{Arc, Mutex};

/// Used to know how to interpret audio data.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct AudioFormat {
    pub channels: u8,
    pub sample_rate: u32,
}

/// Indicates the state of an [`AudioSource`](crate::AudioSource).
#[derive(Debug, Eq, PartialEq)]
pub enum StreamState {
    /// The source had sufficient data to fill the buffer.
    Good,
    /// The source had insufficient data to fill the buffer, but more may come later.
    Underrun,
    /// The source has no more data and doesn't expect more.
    Finished,
}

/// Indicates the amount of data read and the status of an [`AudioSource`](crate::AudioSource).
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

/// Bundles an [`AudioFormat`](crate::AudioFormat) with a buffer containing
/// audio data in that format.
pub struct AudioBuffer<'a> {
    pub samples: &'a mut [Sample],
    pub format: AudioFormat,
}

impl<'a> AudioBuffer<'a> {
    pub fn new(format: AudioFormat, samples: &'a mut [Sample]) -> Self {
        AudioBuffer { format, samples }
    }
}

/// Trait implemented to provide audio data to consumers.
///
/// This is the center of this entire library. Almost everything
/// is either an `AudioSource` or consumes an `AudioSource`.
pub trait AudioSource {
    /// Consume audio data and attempt to fill the given buffer.
    ///
    /// # Returns
    ///
    /// A [`ReadResult`](crate::ReadResult) indicating how much
    /// data was put in the buffer and the state of the source.
    ///
    /// # Panics
    ///
    /// May panic if the format of the buffer is incompatible
    /// with this source or its upstream sources.
    fn read(&mut self, buffer: &mut AudioBuffer) -> ReadResult;
}

pub type SharedAudioSource = Arc<Mutex<dyn AudioSource + Send>>;

/// Helpful extension to move [`AudioSource`](crate::AudioSource) implementations
/// into an `Arc<Mutex<...>>`.
pub trait IntoShared {
    /// Move this audio source into an `Arc<Mutex<...>>` so that it can be
    /// shared between threads.
    fn into_shared(self) -> SharedAudioSource;
}

impl<T: AudioSource + Send + 'static> IntoShared for T {
    fn into_shared(self) -> SharedAudioSource {
        Arc::new(Mutex::new(self))
    }
}
