#[macro_use]
extern crate criterion;
extern crate stl_thumb;

use criterion::Criterion;
use stl_thumb::Config;

fn cube() {
    let config = Config {
        stl_filename: "cube.stl".to_string(),
        img_filename: "cube.png".to_string(),
    };

    stl_thumb::run(&config).expect("Error in run function");
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("cube", |b| b.iter(|| cube()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
