use criterion::{Criterion, criterion_group, criterion_main};
use std::{hint::black_box, time::Duration};
use tarts::{
    common::{self, TerminalEffect},
    rain::{digital_rain, rain_drop},
};

fn get_sane_options() -> digital_rain::DigitalRainOptions {
    digital_rain::DigitalRainOptionsBuilder::default()
        .drops_range((10, 20))
        .speed_range((2, 16))
        .build()
        .unwrap()
}

fn run_loop_benchmark(_c: &mut Criterion) {
    let mut cc = Criterion::default()
        .warm_up_time(std::time::Duration::from_secs(3)) // 3 seconds warm-up time
        .measurement_time(std::time::Duration::from_secs(20)) // 10 seconds measurement time
        .sample_size(100);

    cc.bench_function("benchmark_run_loop", |b| {
        let mut stdout = Vec::new();
        let options = get_sane_options();
        let mut rain = digital_rain::DigitalRain::new(options, (80, 40));

        b.iter(|| {
            let _ = common::run_loop(black_box(&mut stdout), &mut rain, Some(3));
        })
    });
}

fn vertical_worm_benchmark(c: &mut Criterion) {
    let options = get_sane_options();
    c.bench_function("benchmark_raindrop_new_1000", |b| {
        b.iter(|| {
            let mut rng = rand::rng();
            for index in 1..=1000 {
                rain_drop::RainDrop::new((80, 40), &options, index, &mut rng);
            }
        })
    });

    c.bench_function("benchmark_raindrop_update_1000", |b| {
        let mut rng = rand::rng();
        let options = get_sane_options();
        let mut drops: Vec<rain_drop::RainDrop> = vec![];
        for index in 1..=1000 {
            drops.push(rain_drop::RainDrop::new(
                (80, 40),
                &options,
                index,
                &mut rng,
            ));
        }
        b.iter(|| {
            for drop in drops.iter_mut() {
                drop.update(
                    (80, 40),
                    &options,
                    Duration::from_millis(50),
                    &mut rng,
                );
            }
        })
    });
}

fn digital_rain_benchmark(c: &mut Criterion) {
    c.bench_function("benchmark_rain_new", |b| {
        b.iter(|| {
            let options = get_sane_options();
            let _ = digital_rain::DigitalRain::new(options, (80, 40));
        })
    });

    c.bench_function("benchmark_rain_update", |b| {
        b.iter(|| {
            let options = get_sane_options();
            let mut rain = digital_rain::DigitalRain::new(options, (80, 40));
            for _ in 1..=10 {
                rain.update();
            }
        })
    });
}

criterion_group!(
    benches,
    run_loop_benchmark,
    vertical_worm_benchmark,
    digital_rain_benchmark
);
criterion_main!(benches);
