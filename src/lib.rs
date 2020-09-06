mod core;
pub use crate::core::{AudioBuffer, AudioFormat, AudioSource, IntoShared, ReadResult, StreamState};

pub mod decoders;
pub mod drivers;
pub mod effects;
pub mod generators;

pub mod prelude;

mod sdl_util;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
