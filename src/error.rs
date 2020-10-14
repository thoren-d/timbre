/// Unified error type.
#[derive(Debug)]
pub enum Error {
    /// Wraps a [`std::io::Error`](std::io::Error), such as file not found, etc.
    IoError(std::io::Error),
    /// Wraps an error from SDL2.
    SdlError(String),
}

impl Error {
    pub(crate) fn from_sdl(err: String) -> Error {
        Error::SdlError(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}
