use crate::{
    core::{AudioBuffer, SharedAudioSource},
    AudioFormat, AudioSource, ReadResult,
};

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use tracing::{info, instrument};

/// A source for audio captured by a microphone, etc.
///
/// # Examples
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use timbre::drivers::{Sdl2Input, Sdl2Output};
/// let sdl = sdl2::init()?;
/// let audio = sdl.audio()?;
///
/// let mut microphone = Sdl2Input::new(&audio);
/// let mut speaker = Sdl2Output::new(&audio);
/// microphone.resume();
/// speaker.set_source(microphone.source());
/// speaker.resume();
/// # Ok(())
/// # }
/// ```
pub struct Sdl2Input {
    device: sdl2::audio::AudioDevice<Callback>,
    buffer: Arc<Mutex<VecDeque<f32>>>,
}

struct Callback {
    pub format: AudioFormat,
    pub buffer: Arc<Mutex<VecDeque<f32>>>,
}

struct AudioSourceImpl {
    pub format: AudioFormat,
    pub buffer: Arc<Mutex<VecDeque<f32>>>,
}

impl AudioCallback for Callback {
    type Channel = f32;
    #[instrument(name = "Sdl2Input::callback", skip(self, samples))]
    fn callback(&mut self, samples: &mut [Self::Channel]) {
        self.buffer.lock().unwrap().extend(samples.iter().cloned());
    }
}

impl Sdl2Input {
    /// Construct a new `Sdl2Input` with the default format.
    ///
    /// The default format is stereo at 44.1 kHz.
    ///
    /// # Arguments
    ///
    /// * `subsystem` -- An SDL [`AudioSubystem`](sdl2::AudioSubsystem) used to create a capture device.
    ///
    /// # Panics
    ///
    /// If SDL fails to open the device.
    ///
    /// # Examples
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use timbre::drivers::Sdl2Input;
    /// let sdl = sdl2::init()?;
    /// let audio = sdl.audio()?;
    ///
    /// let microphone = Sdl2Input::new(&audio);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(subsystem: &sdl2::AudioSubsystem) -> Self {
        Sdl2Input::with_format(
            subsystem,
            AudioFormat {
                channels: 2,
                sample_rate: 44100,
            },
        )
    }

    /// Construct a new `Sdl2Input` with the specified format.
    ///
    /// This constructor will request the specified format, but the driver may provide something else.
    ///
    /// # Arguments
    ///
    /// * `subsystem` -- An SDL [`AudioSubystem`](sdl2::AudioSubsystem) used to create a capture device.
    /// * `format` -- The format to request for this input device.
    ///
    /// # Panics
    ///
    /// If SDL fails to open the device.
    ///
    /// # Examples
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use timbre::{AudioFormat, drivers::Sdl2Input};
    /// let sdl = sdl2::init()?;
    /// let audio = sdl.audio()?;
    ///
    /// let format = AudioFormat { channels: 2, sample_rate: 44100 };
    /// let microphone = Sdl2Input::with_format(&audio, format);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_format(subsystem: &sdl2::AudioSubsystem, format: AudioFormat) -> Self {
        let desired_spec = AudioSpecDesired {
            freq: Some(format.sample_rate as i32),
            channels: Some(format.channels),
            samples: Some(1024),
        };

        let buffer = Arc::new(Mutex::new(VecDeque::new()));

        let device = subsystem
            .open_capture(None, &desired_spec, |spec| {
                info!("Input Spec: {:?}", spec);

                Callback {
                    buffer: buffer.clone(),
                    format: spec.into(),
                }
            })
            .unwrap();

        Sdl2Input { device, buffer }
    }

    /// Return the device's chosen format.
    ///
    /// # Returns
    ///
    /// The format SDL chose for this input device, which may be different from the requested format.
    pub fn format(&mut self) -> AudioFormat {
        self.device.lock().format
    }

    /// Get an AudioSource impl that reads from this input device.
    ///
    /// All AudioSource implementations returned by this method consume the same
    /// buffer, so you probably only want one.
    pub fn source(&mut self) -> SharedAudioSource {
        Arc::new(Mutex::new(AudioSourceImpl {
            buffer: Arc::clone(&self.buffer),
            format: self.device.lock().format,
        }))
    }

    /// Start/resume this input device.
    ///
    /// This must be called for the [`Sdl2Input`](crate::drivers::Sdl2Input) to
    /// start populating its buffer.
    pub fn resume(&mut self) {
        self.device.resume();
    }

    /// Pause recording for this input device.
    ///
    /// While paused, the internal buffer will not receive new data, and
    /// eventually any sources created from this device will underrun.
    pub fn pause(&mut self) {
        self.device.pause();
    }
}

impl AudioSource for AudioSourceImpl {
    #[instrument(name = "Sdl2Input::read", skip(self, buffer))]
    fn read(&mut self, buffer: &mut AudioBuffer) -> ReadResult {
        assert!(self.format == buffer.format);
        let samples = &mut buffer.samples;

        let mut buffer = self.buffer.lock().unwrap();

        let mut i: usize = 0;
        while i < samples.len() {
            if let Some(sample) = buffer.pop_front() {
                samples[i] = sample;
            } else {
                return ReadResult::underrun(i);
            }
            i += 1;
        }

        ReadResult::good(samples.len())
    }
}
