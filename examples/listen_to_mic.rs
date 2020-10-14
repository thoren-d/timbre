use std::error::Error;
use timbre::{
    self,
    drivers::{Sdl2Input, Sdl2Output},
};
use tracing_subscriber::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let (chrome_layer, _guard) = tracing_chrome::ChromeLayerBuilder::new().build();
    tracing_subscriber::registry().with(chrome_layer).init();

    let sdl = sdl2::init()?;
    let audio = sdl.audio()?;

    let mut mic = Sdl2Input::new(&audio)?;
    let mut output = Sdl2Output::new(&audio)?;

    mic.resume();
    output.set_source(mic.source());
    output.resume();

    println!("Press enter to exit...");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;

    Ok(())
}
