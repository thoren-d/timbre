use crate::{core::AudioBuffer, AudioFormat, AudioSource, Error, ReadResult};

use sdl2::{
    audio::{AudioFormatNum, AudioSpecWAV},
    rwops::RWops,
};

use std::{convert::TryInto, io::Read};
use tracing::instrument;

/// An AudioSource that reads audio data from a WAV file.
///
/// WavDecoder reads from the given WAV file; when finished, AudioSource::read
/// returns [`Finished`](crate::StreamState::Finished) status.
pub struct WavDecoder {
    data: Vec<f32>,
    format: AudioFormat,
    position: usize,
}

impl WavDecoder {
    /// Construct a WavDecoder that reads from a [`std::io::Read`](std::io::Read).
    ///
    /// # Errors
    ///
    /// If the WAV file in `read` in corrupted or empty, will return the underlying SDL error.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use timbre::decoders::WavDecoder;
    ///
    /// let decoder = WavDecoder::new(std::fs::File::open("./assets/music-mono-f32.wav")?)?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(name = "WavDecoder::new", skip(read))]
    pub fn new<R: Read>(mut read: R) -> Result<Self, Error> {
        let mut read_buffer = Vec::new();
        let mut rwops = RWops::from_read(&mut read, &mut read_buffer).map_err(Error::from_sdl)?;
        let wav_data = AudioSpecWAV::load_wav_rw(&mut rwops).map_err(Error::from_sdl)?;
        let data = convert_samples(wav_data.buffer(), wav_data.format);

        let format = AudioFormat {
            channels: wav_data.channels,
            sample_rate: wav_data.freq as u32,
        };

        Ok(WavDecoder {
            data,
            format,
            position: 0,
        })
    }

    /// Construct a WavDecoder the file given by `path`.
    ///
    /// # Errors
    ///
    /// If the file cannot be opened or is not a valid WAV file, will return the underlying SDL error.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use timbre::decoders::WavDecoder;
    ///
    /// let decoder = WavDecoder::from_file("./assets/music-stereo-i16.wav")?;
    /// # Ok(())
    /// }
    /// ```
    #[instrument(name = "WavDecoder::from_file")]
    pub fn from_file(path: &str) -> Result<Self, Error> {
        let wav_data = AudioSpecWAV::load_wav(path).map_err(Error::from_sdl)?;
        let data = convert_samples(wav_data.buffer(), wav_data.format);

        let format = AudioFormat {
            channels: wav_data.channels,
            sample_rate: wav_data.freq as u32,
        };

        Ok(WavDecoder {
            data,
            format,
            position: 0,
        })
    }
}

impl AudioSource for WavDecoder {
    #[instrument(name = "WavDecoder::read", skip(self, buffer))]
    fn read(&mut self, buffer: &mut AudioBuffer) -> ReadResult {
        assert!(self.format == buffer.format);

        let samples = &mut buffer.samples;
        let remaining = self.data.len() - self.position;

        if samples.len() <= remaining {
            samples.copy_from_slice(&self.data[self.position..self.position + samples.len()]);
            self.position += samples.len();
            ReadResult::good(samples.len())
        } else {
            samples[..remaining].copy_from_slice(&self.data[self.position..self.data.len()]);
            self.position = self.data.len();
            ReadResult::finished(remaining)
        }
    }
}

#[instrument(skip(buffer))]
fn convert_samples(buffer: &[u8], format: sdl2::audio::AudioFormat) -> Vec<f32> {
    match format {
        sdl2::audio::AudioFormat::F32LSB => {
            assert!(buffer.len() % std::mem::size_of::<f32>() == 0);
            buffer
                .chunks_exact(std::mem::size_of::<f32>())
                .map(|data| f32::from_le_bytes(data.try_into().unwrap()))
                .collect()
        }
        sdl2::audio::AudioFormat::F32MSB => {
            assert!(buffer.len() % std::mem::size_of::<f32>() == 0);
            buffer
                .chunks_exact(std::mem::size_of::<f32>())
                .map(|data| f32::from_be_bytes(data.try_into().unwrap()))
                .collect()
        }
        sdl2::audio::AudioFormat::S32LSB => {
            assert!(buffer.len() % std::mem::size_of::<i32>() == 0);
            buffer
                .chunks_exact(std::mem::size_of::<i32>())
                .map(|data| {
                    i32::from_le_bytes(data.try_into().unwrap()) as f32 / std::i32::MAX as f32
                })
                .collect()
        }
        sdl2::audio::AudioFormat::S32MSB => {
            assert!(buffer.len() % std::mem::size_of::<f32>() == 0);
            buffer
                .chunks_exact(std::mem::size_of::<i32>())
                .map(|data| {
                    i32::from_be_bytes(data.try_into().unwrap()) as f32 / std::i32::MAX as f32
                })
                .collect()
        }
        sdl2::audio::AudioFormat::S16LSB => {
            assert!(buffer.len() % std::mem::size_of::<i16>() == 0);
            buffer
                .chunks_exact(std::mem::size_of::<i16>())
                .map(|data| {
                    i16::from_le_bytes(data.try_into().unwrap()) as f32 / std::i16::MAX as f32
                })
                .collect()
        }
        sdl2::audio::AudioFormat::S16MSB => {
            assert!(buffer.len() % std::mem::size_of::<f32>() == 0);
            buffer
                .chunks_exact(std::mem::size_of::<i16>())
                .map(|data| {
                    i16::from_be_bytes(data.try_into().unwrap()) as f32 / std::i16::MAX as f32
                })
                .collect()
        }
        sdl2::audio::AudioFormat::S8 => buffer
            .chunks_exact(std::mem::size_of::<i8>())
            .map(|data| i8::from_ne_bytes(data.try_into().unwrap()) as f32 / std::i8::MAX as f32)
            .collect(),
        sdl2::audio::AudioFormat::U16LSB => {
            assert!(buffer.len() % std::mem::size_of::<u16>() == 0);
            buffer
                .chunks_exact(std::mem::size_of::<u16>())
                .map(|data| {
                    let sample = u16::from_le_bytes(data.try_into().unwrap()) as u16;
                    (sample as f32 - u16::SILENCE as f32) / std::i16::MAX as f32
                })
                .collect()
        }
        sdl2::audio::AudioFormat::U16MSB => {
            assert!(buffer.len() % std::mem::size_of::<u16>() == 0);
            buffer
                .chunks_exact(std::mem::size_of::<u16>())
                .map(|data| {
                    let sample = u16::from_be_bytes(data.try_into().unwrap()) as u16;
                    (sample as f32 - u16::SILENCE as f32) / std::i16::MAX as f32
                })
                .collect()
        }
        sdl2::audio::AudioFormat::U8 => buffer
            .iter()
            .map(|&sample| (sample as f32 - u8::SILENCE as f32) / std::i8::MAX as f32)
            .collect(),
    }
}
