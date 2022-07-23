#[macro_use]
extern crate criterion;
extern crate stl_thumb;

use criterion::Criterion;
use stl_thumb::config::Config;

fn cube() {
    let config = Config {
        stl_filename: "test_data/cube.stl".to_string(),
        img_filename: "cube.png".to_string(),
        width: 1024,
        height: 768,
        ..Default::default()
    };

    stl_thumb::render_to_file(&config).expect("Error in run function");
}

fn cube_obj() {
    let config = Config {
        stl_filename: "test_data/cube.obj".to_string(),
        img_filename: "cube_obj.png".to_string(),
        width: 1024,
        height: 768,
        ..Default::default()
    };

    stl_thumb::render_to_file(&config).expect("Error in run function");
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("cube stl", |b| b.iter(|| cube()));
    c.bench_function("cube obj", |b| b.iter(|| cube_obj()));
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
}
criterion_main!(benches);
