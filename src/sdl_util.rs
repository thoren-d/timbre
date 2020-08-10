use crate::core::AudioFormat;

use sdl2::audio::AudioSpec;

impl From<AudioSpec> for AudioFormat {
    fn from(spec: AudioSpec) -> Self {
        match spec.channels {
            1 => AudioFormat::Mono(spec.freq),
            2 => AudioFormat::Stereo(spec.freq),
            _ => panic!("Unsupported output format."),
        }
    }
}
