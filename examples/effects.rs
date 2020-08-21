use sdl2;
use std::error::Error;
use timbre::{
    decoders::WavDecoder,
    drivers::Sdl2Output,
    effects::{BasicMixer, Echo, HighPass, LowPass},
    Share,
};
use tracing_subscriber::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let (chrome_layer, _guard) = tracing_chrome::ChromeLayerBuilder::new().build();
    tracing_subscriber::registry().with(chrome_layer).init();

    let sdl = sdl2::init()?;
    let audio = sdl.audio()?;

    let track1 = WavDecoder::from_file("./assets/music-stereo-f32.wav");
    let track2 = WavDecoder::from_file("./assets/music-stereo-i16.wav");

    let low_pass = LowPass::new(track1.share(), 300.0);
    let high_pass = HighPass::new(track2.share(), 4000.0);

    let mut mixer = BasicMixer::new(timbre::AudioFormat::Stereo(44100), Some(0.33));
    mixer.add_source(low_pass.share());
    mixer.add_source(high_pass.share());

    let echo = Echo::new(mixer.share(), 30000, 0.7);

    let mut output = Sdl2Output::new(&audio);
    output.set_source(echo.share());
    output.resume();

    println!("Press enter to exit...");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;

    Ok(())
}
