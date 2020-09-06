use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use timbre::{
    effects::{BasicMixer, Echo, HighPass, LowPass},
    prelude::*,
};
use timbre::{generators::SinWave, AudioBuffer, AudioFormat};

const WINDOW_SIZE: usize = 1024;
const SAMPLE_RATE: usize = 44100;
const CHANNELS: usize = 2;

fn bench_echo(c: &mut Criterion) {
    let mut group = c.benchmark_group("Echo");
    for delay in [0.1, 0.5, 1.0, 2.0, 4.0].iter() {
        group.bench_with_input(BenchmarkId::new("read", delay), delay, |b, &delay| {
            let mut samples = Vec::new();
            samples.resize(WINDOW_SIZE * CHANNELS, 0.0);

            let mut buffer = AudioBuffer {
                format: AudioFormat {
                    channels: CHANNELS as u8,
                    sample_rate: SAMPLE_RATE as u32,
                },
                samples: &mut samples[..],
            };

            let sin_wave = SinWave::new(1.0, 440.0);
            let mut echo = Echo::new(
                sin_wave.into_shared(),
                std::time::Duration::from_secs_f32(delay),
                0.5,
            );

            b.iter(|| {
                echo.read(&mut buffer);
            });
            black_box(samples);
        });
    }
}

fn bench_highpass(c: &mut Criterion) {
    let mut group = c.benchmark_group("HighPass");
    for channels in [1, 2].iter() {
        group.bench_with_input(
            BenchmarkId::new("read", channels),
            channels,
            |b, &channels| {
                let mut samples = Vec::new();
                samples.resize(WINDOW_SIZE * channels, 0.0);

                let mut buffer = AudioBuffer {
                    format: AudioFormat {
                        channels: channels as u8,
                        sample_rate: SAMPLE_RATE as u32,
                    },
                    samples: &mut samples[..],
                };

                let sin_wave = SinWave::new(1.0, 440.0);
                let mut high_pass = HighPass::new(sin_wave.into_shared(), 1000.0);
                b.iter(|| {
                    high_pass.read(&mut buffer);
                });
                black_box(samples);
            },
        );
    }
}

fn bench_lowpass(c: &mut Criterion) {
    let mut group = c.benchmark_group("LowPass");
    for channels in [1, 2].iter() {
        group.bench_with_input(
            BenchmarkId::new("read", channels),
            channels,
            |b, &channels| {
                let mut samples = Vec::new();
                samples.resize(WINDOW_SIZE * channels, 0.0);

                let mut buffer = AudioBuffer {
                    format: AudioFormat {
                        channels: channels as u8,
                        sample_rate: SAMPLE_RATE as u32,
                    },
                    samples: &mut samples[..],
                };

                let sin_wave = SinWave::new(1.0, 440.0);
                let mut high_pass = LowPass::new(sin_wave.into_shared(), 1000.0);
                b.iter(|| {
                    high_pass.read(&mut buffer);
                });
                black_box(samples);
            },
        );
    }
}

fn bench_basicmixer(c: &mut Criterion) {
    let mut group = c.benchmark_group("BasicMixer");
    for sources in [1, 2, 4, 8, 16].iter() {
        group.bench_with_input(BenchmarkId::new("read", sources), sources, |b, &sources| {
            let mut samples = Vec::new();
            samples.resize(WINDOW_SIZE * CHANNELS, 0.0);

            let mut buffer = AudioBuffer {
                format: AudioFormat {
                    channels: CHANNELS as u8,
                    sample_rate: SAMPLE_RATE as u32,
                },
                samples: &mut samples[..],
            };

            let sin_wave = SinWave::new(1.0, 440.0);
            let mut basic_mixer = BasicMixer::new();

            for _ in 0..sources {
                basic_mixer.add_source(sin_wave.clone().into_shared());
            }

            b.iter(|| {
                basic_mixer.read(&mut buffer);
            });
            black_box(samples);
        });
    }
}

criterion_group!(
    benches,
    bench_echo,
    bench_highpass,
    bench_lowpass,
    bench_basicmixer
);
criterion_main!(benches);
