use crate::core::AudioFormat;

use sdl2::audio::AudioSpec;

impl From<AudioSpec> for AudioFormat {
    fn from(spec: AudioSpec) -> Self {
        AudioFormat {
            channels: spec.channels,
            sample_rate: spec.freq as u32,
        }
    }
}
