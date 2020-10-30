timbre
======

[![Crates.io](https://img.shields.io/crates/v/timbre)](https://crates.io/crates/timbre)
[![Documentation](https://docs.rs/timbre/badge.svg)](https://docs.rs/timbre/)
![GitHub](https://img.shields.io/github/license/thoren-d/timbre)
![CI](https://github.com/thoren-d/timbre/workflows/CI/badge.svg?branch=develop)

timbre is an audio library designed for composing real-time effects.

Timbre is designed to establish a common interface and a decent-sized
library of effects and decoders for playing audio in real time. It is aimed
at eventually providing most of the audio functionality needed for game
programming, but should be flexible enough for other applications as well.

# Example

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;
    use timbre::prelude::*;

    // SDL setup.
    let sdl = sdl2::init()?;
    let audio = sdl.audio()?;

    // Inputs
    let mut microphone = timbre::drivers::Sdl2Input::new(&audio)?;
    microphone.resume();
    let music = timbre::decoders::WavDecoder::from_file("./assets/music-stereo-f32.wav")?;

    // Apply effects
    let microphone = timbre::effects::Echo::new(microphone.source(),
            Duration::from_secs_f32(0.5), 0.6);
    let music = timbre::effects::LowPass::new(music, 200.0);

    // Mix them together
    let mut mixer = timbre::effects::BasicMixer::new();
    mixer.add_source(microphone.into_shared());
    mixer.add_source(music.into_shared());

    // Output
    let mut speaker = timbre::drivers::Sdl2Output::new(&audio)?;
    speaker.set_source(mixer.into_shared());
    speaker.resume();

    std::thread::sleep(Duration::from_secs_f32(10.0));
    Ok(())
}
```

# What's new in 0.3?

* Functions that can fail now return Result.
* New Error type.

# Roadmap

## 0.4

* Make effects generic to reduce number of `Arc<Mutex<...>>`.
* Make effects more mutable; give access to their source.

## 0.5

* Introduce 3D spatialization effect.
* Add Pan effect for stereo sources.
