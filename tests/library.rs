//! End-to-end tests of the library API as used by long-running callers such as
//! file manager thumbnail plugins (e.g. stl-thumb-kde), which render many models
//! from a single process.

use std::ffi::CString;
use std::thread;
use stl_thumb::config::Config;

const MODELS: [&str; 3] = [
    "test_data/cube.stl",
    "test_data/cube.obj",
    "test_data/cube.3mf",
];

/// Number of opaque pixels with the default (blue) material. Zero means nothing was drawn.
fn model_pixels(rgba: &[u8]) -> usize {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] == 255 && p[2] > 150)
        .count()
}

fn config(model: &str, size: u32) -> Config {
    Config {
        model_filename: model.to_string(),
        width: size,
        height: size,
        ..Default::default()
    }
}

#[test]
fn render_to_image_repeatedly_in_one_process() {
    for round in 0..3 {
        for (i, model) in MODELS.iter().enumerate() {
            let size = 128 + 64 * i as u32;
            let img = stl_thumb::render_to_image(&config(model, size))
                .unwrap_or_else(|e| panic!("round {round}, {model}: {e}"));
            assert_eq!((img.width(), img.height()), (size, size));
            let rgba = img.to_rgba8();
            assert!(
                model_pixels(&rgba) > 0,
                "round {round}, {model}: rendered image is blank"
            );
        }
    }
}

#[test]
fn render_to_image_from_many_threads() {
    let handles: Vec<_> = (0..6)
        .map(|i| {
            thread::spawn(move || {
                let model = MODELS[i % MODELS.len()];
                let img = stl_thumb::render_to_image(&config(model, 96)).expect("render failed");
                model_pixels(&img.to_rgba8())
            })
        })
        .collect();
    for h in handles {
        assert!(h.join().expect("thread panicked") > 0);
    }
}

#[test]
fn c_api_render_to_buffer_repeatedly() {
    for model in MODELS.iter().cycle().take(6) {
        let (w, h) = (160u32, 120u32);
        // Fill with a pattern so leftover (unrendered) memory is detectable
        let mut buf = vec![0xABu8; (w * h * 4) as usize];
        let path = CString::new(*model).unwrap();
        let ok = unsafe { stl_thumb::render_to_buffer(buf.as_mut_ptr(), w, h, path.as_ptr()) };
        assert!(ok, "{model}: render_to_buffer returned false");
        assert!(
            model_pixels(&buf) > 0,
            "{model}: buffer has no rendered model"
        );
    }
}

#[test]
fn c_api_reports_missing_file() {
    let mut buf = vec![0u8; 64 * 64 * 4];
    let path = CString::new("test_data/does-not-exist.stl").unwrap();
    let ok = unsafe { stl_thumb::render_to_buffer(buf.as_mut_ptr(), 64, 64, path.as_ptr()) };
    assert!(!ok);

    // A failed call must not break later calls
    let path = CString::new("test_data/cube.stl").unwrap();
    let ok = unsafe { stl_thumb::render_to_buffer(buf.as_mut_ptr(), 64, 64, path.as_ptr()) };
    assert!(ok);
    assert!(model_pixels(&buf) > 0);
}
