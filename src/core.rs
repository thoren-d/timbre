use std::sync::{Arc, Mutex};

pub enum AudioFormat {
    Mono(i32),
    Stereo(i32),
}

impl Clone for AudioFormat {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for AudioFormat {}

impl PartialEq for AudioFormat {
    fn eq(&self, other: &Self) -> bool {
        use AudioFormat::{Mono, Stereo};
        match self {
            Mono(n) => {
                if let Mono(p) = other {
                    p == n
                } else {
                    false
                }
            }
            Stereo(n) => {
                if let Stereo(p) = other {
                    p == n
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum StreamState {
    Good,
    Underrun(usize),
    Finished(usize),
}

pub trait AudioSource {
    fn format(&mut self) -> AudioFormat;
    fn read(&mut self, samples: &mut [f32]) -> StreamState;
}

pub trait Share {
    fn share(self) -> Arc<Mutex<Self>>;
}

impl<T: AudioSource + Send> Share for T {
    fn share(self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(self))
    }
}
