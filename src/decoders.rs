//! [`AudioSource`](crate::AudioSource) implementations that read common audio codecs.

mod wav_decoder;

pub use wav_decoder::WavDecoder;
