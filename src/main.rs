use timbre::decoders::WavDecoder;
use timbre::drivers::{Sdl2Input, Sdl2Output};
use timbre::effects::{BasicMixer, Echo, LowPass};
use timbre::Share;

use std::sync::{Arc, Mutex};

fn main() {
    let sdl_context = sdl2::init().unwrap();
    for driver in sdl2::audio::drivers() {
        println!("{:?}", driver);
    }
    let audio_subsystem = sdl_context.audio().unwrap();
    println!("Driver: {}", audio_subsystem.current_audio_driver());
    for i in 0..audio_subsystem.num_audio_playback_devices().unwrap() {
        println!(
            "Audio device: {}",
            audio_subsystem.audio_playback_device_name(i).unwrap()
        );
    }

    let microphone = Arc::new(Mutex::new(Sdl2Input::new(&audio_subsystem)));
    microphone.lock().unwrap().resume();
    let mic_echo = Arc::new(Mutex::new(Echo::new(
        microphone.lock().unwrap().source(),
        44100,
        0.5,
    )));

    let music = WavDecoder::from_file("./sample2-f32.wav");
    let music = LowPass::new(music.share(), 100.0);

    let mut mixer = BasicMixer::new(timbre::AudioFormat::Stereo(44100), Some(1.0));
    mixer.add_source(music.share());
    mixer.add_source(mic_echo);

    let mut output = Sdl2Output::new(&audio_subsystem);
    output.set_source(mixer.share());
    output.resume();

    std::thread::sleep(std::time::Duration::from_secs(3600));
}
