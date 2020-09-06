//! Sources and sinks that connect to hardware.

mod sdl2_input;
mod sdl2_output;

pub use sdl2_input::Sdl2Input;
pub use sdl2_output::Sdl2Output;
