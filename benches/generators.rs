use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use timbre::prelude::*;
use timbre::{generators::SinWave, AudioBuffer, AudioFormat};

const WINDOW_SIZE: usize = 1024;
const SAMPLE_RATE: usize = 44100;

fn bench_sinwave(c: &mut Criterion) {
    let mut group = c.benchmark_group("SinWav");
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

                let mut sin_wave = SinWave::new(1.0, 440.0);

                b.iter(|| {
                    sin_wave.read(&mut buffer);
                });
                black_box(samples);
            },
        );
    }
}

criterion_group!(benches, bench_sinwave);
criterion_main!(benches);
