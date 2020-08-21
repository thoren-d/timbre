use sdl2;
use timbre::{self, drivers::{Sdl2Output, Sdl2Input}, };
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let sdl = sdl2::init()?;
    let audio = sdl.audio()?;

    let mut mic = Sdl2Input::new(&audio);
    let mut output = Sdl2Output::new(&audio);

    mic.resume();
    output.set_source(mic.source());
    output.resume();

    println!("Press enter to exit...");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;

    Ok(())
}
