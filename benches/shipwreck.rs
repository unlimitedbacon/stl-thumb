#[macro_use]
extern crate criterion;
extern crate stl_thumb;

use criterion::Criterion;
use stl_thumb::config::Config;

fn shipwreck() {
    let config = Config {
        stl_filename: "test_data/shipwreck.stl".to_string(),
        img_filename: "shipwreck.png".to_string(),
        width: 1024,
        height: 768,
        ..Default::default()
    };

    stl_thumb::render_to_file(&config).expect("Error in run function");
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("shipwreck", |b| b.iter(|| shipwreck()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
