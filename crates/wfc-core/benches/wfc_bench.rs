//! Benchmarks for wfc-core hot paths.

use criterion::{Criterion, criterion_group, criterion_main};
use wfc_core::{Config, Sample, StepOutcome, Wfc, default_pipe_sample};

fn large_sample() -> Sample {
    // 16x16 sample with varied colors for a richer pattern set
    let mut pixels = Vec::with_capacity(16 * 16);
    for y in 0..16u8 {
        for x in 0..16u8 {
            let r = (x.wrapping_mul(37).wrapping_add(y.wrapping_mul(13))) % 4 * 64;
            let g = (x.wrapping_mul(11).wrapping_add(y.wrapping_mul(29))) % 4 * 64;
            let b = (x.wrapping_mul(23).wrapping_add(y.wrapping_mul(7))) % 4 * 64;
            pixels.push([r, g, b]);
        }
    }
    Sample::new(16, 16, pixels)
}

fn bench_run(c: &mut Criterion) {
    let pipes = default_pipe_sample();
    let large = large_sample();

    c.bench_function("run_32x32_pipes", |b| {
        let config = Config {
            seed: Some(42),
            output_width: 32,
            output_height: 32,
            ..Default::default()
        };
        b.iter(|| {
            let mut wfc = Wfc::new(&pipes, config.clone());
            wfc.run();
        });
    });

    c.bench_function("run_64x64_pipes", |b| {
        let config = Config {
            seed: Some(42),
            output_width: 64,
            output_height: 64,
            ..Default::default()
        };
        b.iter(|| {
            let mut wfc = Wfc::new(&pipes, config.clone());
            wfc.run();
        });
    });

    c.bench_function("run_32x32_large_sample", |b| {
        let config = Config {
            seed: Some(42),
            output_width: 32,
            output_height: 32,
            backtracking: true,
            ..Default::default()
        };
        b.iter(|| {
            let mut wfc = Wfc::new(&large, config.clone());
            wfc.run();
        });
    });
}

fn bench_init(c: &mut Criterion) {
    let pipes = default_pipe_sample();
    let large = large_sample();

    c.bench_function("init_32x32_pipes", |b| {
        let config = Config {
            seed: Some(42),
            output_width: 32,
            output_height: 32,
            ..Default::default()
        };
        b.iter(|| Wfc::new(&pipes, config.clone()));
    });

    c.bench_function("init_64x64_large_sample", |b| {
        let config = Config {
            seed: Some(42),
            output_width: 64,
            output_height: 64,
            ..Default::default()
        };
        b.iter(|| Wfc::new(&large, config.clone()));
    });
}

fn bench_step(c: &mut Criterion) {
    let pipes = default_pipe_sample();

    c.bench_function("step_100_32x32_pipes", |b| {
        let config = Config {
            seed: Some(42),
            output_width: 32,
            output_height: 32,
            ..Default::default()
        };
        b.iter(|| {
            let mut wfc = Wfc::new(&pipes, config.clone());
            for _ in 0..100 {
                if wfc.step() != StepOutcome::Progressed {
                    break;
                }
            }
        });
    });
}

fn bench_render(c: &mut Criterion) {
    let pipes = default_pipe_sample();
    let config = Config {
        seed: Some(42),
        output_width: 64,
        output_height: 64,
        ..Default::default()
    };
    let mut wfc = Wfc::new(&pipes, config);
    wfc.run();

    c.bench_function("render_64x64", |b| {
        b.iter(|| wfc.render());
    });
}

criterion_group!(benches, bench_run, bench_init, bench_step, bench_render);
criterion_main!(benches);
