#![doc = include_str!("../README.md")]

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
