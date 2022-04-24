#[macro_use]
extern crate criterion;
extern crate stl_thumb;

use criterion::Criterion;
use stl_thumb::config::Config;

fn cube() {
    let config = Config {
        stl_filename: "test_data/cube.stl".to_string(),
        img_filename: Some("cube.png".to_string()),
        width: 1024,
        height: 768,
        ..Default::default()
    };

    stl_thumb::render_to_file(&config).expect("Error in run function");
}

fn benchy() {
    let config = Config {
        stl_filename: "test_data/3DBenchy.stl".to_string(),
        img_filename: Some("benchy.png".to_string()),
        width: 1024,
        height: 768,
        ..Default::default()
    };

    stl_thumb::render_to_file(&config).expect("Error in run function");
}


fn shipwreck() {
    let config = Config {
        stl_filename: "test_data/shipwreck.stl".to_string(),
        img_filename: Some("shipwreck.png".to_string()),
        width: 1024,
        height: 768,
        ..Default::default()
    };

    stl_thumb::render_to_file(&config).expect("Error in run function");
}


fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("cube", |b| b.iter(|| cube()));
    c.bench_function("benchy", |b| b.iter(|| benchy()));
    c.bench_function("shipwreck", |b| b.iter(|| shipwreck()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
