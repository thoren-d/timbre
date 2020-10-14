use crate::{
    core::{AudioBuffer, SharedAudioSource},
    AudioFormat, Error, StreamState,
};

use sdl2::audio::{AudioCallback, AudioFormatNum, AudioSpecDesired};
use tracing::{info, instrument, warn};

struct Callback {
    pub format: AudioFormat,
    pub source: Option<SharedAudioSource>,
}

impl AudioCallback for Callback {
    type Channel = f32;
    #[instrument(name = "Sdl2Output::callback", skip(self, samples))]
    fn callback(&mut self, samples: &mut [Self::Channel]) {
        if let Some(source) = &self.source {
            let mut source = source.lock().unwrap();

            let mut buffer = AudioBuffer::new(self.format, samples);
            let result = source.read(&mut buffer);

            if result.state == StreamState::Underrun {
                warn!("Underrun detected.");
            }

            samples
                .iter_mut()
                .skip(result.read)
                .for_each(|s| *s = AudioFormatNum::SILENCE);
        } else {
            for sample in samples.iter_mut() {
                *sample = AudioFormatNum::SILENCE;
            }
        }
    }
}

/// A sink that outputs audio data to speakers, etc.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use timbre::drivers::{Sdl2Input, Sdl2Output};
/// # std::env::set_var("SDL_AUDIODRIVER", "dummy");
/// let sdl = sdl2::init()?;
/// let audio = sdl.audio()?;
///
/// let mut microphone = Sdl2Input::new(&audio)?;
/// let mut speaker = Sdl2Output::new(&audio)?;
/// microphone.resume();
/// speaker.set_source(microphone.source());
/// speaker.resume();
/// # Ok(())
/// # }
/// ```
pub struct Sdl2Output {
    device: sdl2::audio::AudioDevice<Callback>,
}

impl Sdl2Output {
    /// Construct a new `Sdl2Output` with the default format.
    ///
    /// The default format is stereo at 44.1 kHz.
    ///
    /// # Arguments
    ///
    /// * `subsystem` -- An SDL [`AudioSubystem`](sdl2::AudioSubsystem) used to create an output device.
    ///
    /// # Errors
    ///
    /// If SDL fails to open the device.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use timbre::drivers::Sdl2Output;
    /// # std::env::set_var("SDL_AUDIODRIVER", "dummy");
    /// let sdl = sdl2::init()?;
    /// let audio = sdl.audio()?;
    ///
    /// let speaker = Sdl2Output::new(&audio)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(subsystem: &sdl2::AudioSubsystem) -> Result<Self, Error> {
        Sdl2Output::with_format(
            subsystem,
            AudioFormat {
                channels: 2,
                sample_rate: 44100,
            },
        )
    }

    /// Construct a new `Sdl2Output` with the specified format.
    ///
    /// This constructor will request the specified format, but the driver may choose something else.
    ///
    /// # Arguments
    ///
    /// * `subsystem` -- An SDL [`AudioSubystem`](sdl2::AudioSubsystem) used to create an output device.
    /// * `format` -- The format to request for this output device.
    ///
    /// # Errors
    ///
    /// If SDL fails to open the device.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use timbre::drivers::Sdl2Output;
    /// # std::env::set_var("SDL_AUDIODRIVER", "dummy");
    /// let sdl = sdl2::init()?;
    /// let audio = sdl.audio()?;
    ///
    /// let speaker = Sdl2Output::new(&audio)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_format(
        subsystem: &sdl2::AudioSubsystem,
        format: AudioFormat,
    ) -> Result<Self, Error> {
        let desired_spec = AudioSpecDesired {
            freq: Some(format.sample_rate as i32),
            channels: Some(format.channels),
            samples: Some(1024),
        };

        let device = subsystem
            .open_playback(None, &desired_spec, |spec| {
                info!("Output Spec: {:?}", spec);

                Callback {
                    format: spec.into(),
                    source: None,
                }
            })
            .map_err(Error::from_sdl)?;

        Ok(Sdl2Output { device })
    }

    /// Set the source of audio to output.
    pub fn set_source(&mut self, source: SharedAudioSource) {
        self.device.lock().source = Some(source);
    }

    /// Get the driver's chosen audio format.
    pub fn format(&mut self) -> AudioFormat {
        self.device.lock().format
    }

    /// Pause playback for this device.
    ///
    /// While paused, this device will not consume data from its source.
    pub fn pause(&mut self) {
        self.device.pause();
    }

    /// Start/resume playback for this device.
    ///
    /// The device starts in the paused state, and must be resumed for
    /// playback from an audio source to begin.
    pub fn resume(&mut self) {
        self.device.resume();
    }
}
