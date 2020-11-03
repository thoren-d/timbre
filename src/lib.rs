//! An audio library designed for composing real-time effects.
//!
//! Timbre is designed to establish a common interface and a decent-sized
//! library of effects and decoders for playing audio in real time. It is aimed
//! at eventually providing most of the audio functionality needed for game
//! programming, but should be flexible enough for other applications as well.
//!
//! # Example
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # std::env::set_var("SDL_AUDIODRIVER", "dummy");
//! use std::time::Duration;
//! use timbre::prelude::*;
//!
//! // SDL setup.
//! let sdl = sdl2::init()?;
//! let audio = sdl.audio()?;
//!
//! // Inputs
//! let mut microphone = timbre::drivers::Sdl2Input::new(&audio)?;
//! microphone.resume();
//! let music = timbre::decoders::WavDecoder::from_file("./assets/music-stereo-f32.wav")?;
//!
//! // Apply effects
//! let microphone = timbre::effects::Echo::new(microphone.source(),
//!         Duration::from_secs_f32(0.5), 0.6);
//! let music = timbre::effects::LowPass::new(music, 200.0);
//!
//! // Mix them together
//! let mut mixer = timbre::effects::BasicMixer::new();
//! mixer.add_source(microphone.into_shared());
//! mixer.add_source(music.into_shared());
//!
//! // Output
//! let mut speaker = timbre::drivers::Sdl2Output::new(&audio)?;
//! speaker.set_source(mixer.into_shared());
//! speaker.resume();
//!
//! # Ok(())
//! # }

mod core;
pub use crate::core::*;
mod error;
pub use crate::error::*;

pub mod decoders;
pub mod drivers;
pub mod effects;
pub mod generators;

pub mod prelude;

mod sdl_util;
