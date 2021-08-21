use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use timbre::AudioFormat;
use timbre::{
    effects::{BasicMixer, Echo, HighPass, LowPass},
    prelude::*,
    Sample,
};

const WINDOW_SIZE: usize = 1024;
const SAMPLE_RATE: usize = 44100;
const CHANNELS: usize = 2;

#[derive(Clone)]
struct DummySource {
    number: f32,
}

impl AudioSource for DummySource {
    fn read(&mut self, buffer: &mut [Sample]) -> timbre::ReadResult {
        buffer.iter_mut().for_each(|sample| *sample = self.number);
        timbre::ReadResult::good(buffer.len())
    }

    fn format(&self) -> AudioFormat {
        AudioFormat {
            sample_rate: SAMPLE_RATE as u32,
            channels: CHANNELS as u8,
        }
    }
}

fn bench_echo(c: &mut Criterion) {
    let mut group = c.benchmark_group("Echo");
    for delay in [0.1, 0.5, 1.0, 2.0, 4.0].iter() {
        group.bench_with_input(BenchmarkId::new("read", delay), delay, |b, &delay| {
            let mut samples = Vec::new();
            samples.resize(WINDOW_SIZE * CHANNELS, 0.0);

            let source = DummySource {
                number: black_box(0.5),
            };
            let mut echo = Echo::new(source, Duration::from_secs_f32(delay), 0.5);

            b.iter(|| {
                echo.read(&mut samples);
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

                let source = DummySource {
                    number: black_box(0.5),
                };
                let mut high_pass = HighPass::new(source, 1000.0);
                b.iter(|| {
                    high_pass.read(&mut samples);
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

                let source = DummySource {
                    number: black_box(0.5),
                };
                let mut low_pass = LowPass::new(source, 1000.0);
                b.iter(|| {
                    low_pass.read(&mut samples);
                });
                black_box(samples);
            },
        );
    }
}

fn bench_composite(c: &mut Criterion) {
    let mut group = c.benchmark_group("CompositeEffect");
    for channels in [1, 2].iter() {
        group.bench_with_input(
            BenchmarkId::new("read", channels),
            channels,
            |b, &channels| {
                let mut samples = Vec::new();
                samples.resize(WINDOW_SIZE * channels, 0.0);

                let source = DummySource {
                    number: black_box(0.5),
                };
                let source = LowPass::new(source, 1000.0);
                let source = HighPass::new(source, 100.0);
                let source = Echo::new(source, Duration::from_secs_f32(0.2), 0.5);
                let source = Echo::new(source, Duration::from_secs_f32(0.1), 0.5);
                let mut source = Echo::new(source, Duration::from_secs_f32(0.05), 0.5);
                b.iter(|| {
                    source.read(&mut samples);
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

            let source = DummySource {
                number: black_box(0.5),
            };
            let mut basic_mixer = BasicMixer::new();

            for _ in 0..sources {
                basic_mixer.add_source(source.clone().into_shared());
            }

            b.iter(|| {
                basic_mixer.read(&mut samples);
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
    bench_basicmixer,
    bench_composite
);
criterion_main!(benches);
