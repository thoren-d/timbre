use std::{error::Error, time::Duration};
use timbre::{
    decoders::WavDecoder,
    drivers::Sdl2Output,
    effects::{BasicMixer, Echo, HighPass, LowPass},
    IntoShared,
};
use tracing_subscriber::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let (chrome_layer, _guard) = tracing_chrome::ChromeLayerBuilder::new().build();
    tracing_subscriber::registry().with(chrome_layer).init();

    let sdl = sdl2::init()?;
    let audio = sdl.audio()?;

    let track1 = WavDecoder::from_file("./assets/music-stereo-f32.wav");
    let track2 = WavDecoder::new(std::fs::File::open("./assets/music-stereo-i16.wav")?);

    let low_pass = LowPass::new(track1.into_shared(), 300.0);
    let high_pass = HighPass::new(track2.into_shared(), 4000.0);

    let mut mixer = BasicMixer::new();
    mixer.add_source(low_pass.into_shared());
    mixer.add_source(high_pass.into_shared());

    let echo = Echo::new(mixer.into_shared(), Duration::from_secs_f32(0.5), 0.7);

    let mut output = Sdl2Output::new(&audio);
    output.set_source(echo.into_shared());
    output.resume();

    println!("Press enter to exit...");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;

    Ok(())
}
