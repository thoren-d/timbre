use std::sync::{Arc, Mutex};

/// Used to know how to interpret audio data.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct AudioFormat {
    pub channels: u8,
    pub sample_rate: u32,
}

impl AudioFormat {
    /// Single-channel (mono) audio with CD quality sample rate (44.1 kHz).
    pub const MONO_CD: AudioFormat = AudioFormat {
        channels: 1,
        sample_rate: 44100,
    };

    /// Single-channel (mono) audio with DVD quality sample rate (48 kHz).
    pub const MONO_DVD: AudioFormat = AudioFormat {
        channels: 1,
        sample_rate: 48000,
    };

    /// Two-channel (stereo) audio with CD quality sample rate (44.1 kHz).
    pub const STEREO_CD: AudioFormat = AudioFormat {
        channels: 2,
        sample_rate: 44100,
    };

    /// Two-channel (stereo) audio with DVD quality sample rate (48 kHz).
    pub const STEREO_DVD: AudioFormat = AudioFormat {
        channels: 2,
        sample_rate: 48000,
    };
}

impl Default for AudioFormat {
    /// Returns the default [`AudioFormat`](crate::AudioFormat), currently
    /// [`AudioFormat::STEREO_CD`](crate::AudioFormat::STEREO_CD).
    fn default() -> Self {
        AudioFormat::STEREO_CD
    }
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

/// Trait implemented to provide audio data to consumers.
///
/// This is the center of this entire library. Almost everything
/// is either an `AudioSource` or consumes an `AudioSource`.
pub trait AudioSource {
    /// Returns the format used by this audio source.
    ///
    /// # Returns
    ///
    /// An [`AudioFormat`](crate::AudioFormat) describing the sample rate
    /// and number of channels provided by this `AudioSource`.
    fn format(&self) -> AudioFormat;

    /// Consume audio data and attempt to fill the given buffer.
    ///
    /// # Returns
    ///
    /// A [`ReadResult`](crate::ReadResult) indicating how much
    /// data was put in the buffer and the state of the source.
    ///
    /// # Panics
    ///
    /// May panic if `buffer.len()` is not a multiple of `format().channels`.
    fn read(&mut self, buffer: &mut [Sample]) -> ReadResult;
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

impl AudioSource for SharedAudioSource {
    fn format(&self) -> AudioFormat {
        self.lock().unwrap().format()
    }

    fn read(&mut self, buffer: &mut [Sample]) -> ReadResult {
        self.lock().unwrap().read(buffer)
    }
}
