//! End-to-end tests of the `stl-thumb` command line program.

use std::path::PathBuf;
use std::process::{Command, Output};

fn stl_thumb() -> Command {
    Command::new(env!("CARGO_BIN_EXE_stl-thumb"))
}

fn out_path(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cli");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    let _ = std::fs::remove_file(&path);
    path
}

fn run(cmd: &mut Command) -> Output {
    let out = cmd.output().expect("failed to run stl-thumb");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("Creating EventLoop multiple times"),
        "stl-thumb created a second event loop:\n{stderr}"
    );
    out
}

fn assert_success(out: &Output) {
    assert!(
        out.status.success(),
        "stl-thumb failed ({}):\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Count of pixels that look like the default blue material.
fn model_pixels(img: &image::DynamicImage) -> usize {
    img.to_rgb8()
        .pixels()
        .filter(|p| p[2] > 150 && p[2] > p[0])
        .count()
}

#[test]
fn renders_png_for_each_model_format() {
    for model in ["cube.stl", "cube.obj", "cube.3mf"] {
        let out = out_path(&format!("{model}.png"));
        assert_success(&run(stl_thumb()
            .args(["-s", "200"])
            .arg(format!("test_data/{model}"))
            .arg(&out)));
        let img = image::open(&out).expect("output is not a readable image");
        assert_eq!((img.width(), img.height()), (200, 200));
        assert!(model_pixels(&img) > 0, "{model}: thumbnail is blank");
    }
}

#[test]
fn renders_jpeg() {
    for name in ["cube.jpg", "cube.JPEG"] {
        let out = out_path(name);
        assert_success(&run(stl_thumb().arg("test_data/cube.stl").arg(&out)));
        let bytes = std::fs::read(&out).unwrap();
        assert_eq!(
            image::guess_format(&bytes).unwrap(),
            image::ImageFormat::Jpeg
        );
        let img = image::load_from_memory(&bytes).unwrap();
        assert!(model_pixels(&img) > 0, "{name}: thumbnail is blank");
    }
}

#[test]
fn writes_to_stdout_with_explicit_format() {
    let out = run(stl_thumb().args(["-f", "png", "-s", "64", "test_data/cube.stl", "-"]));
    assert_success(&out);
    let img = image::load_from_memory(&out.stdout).expect("stdout is not an image");
    assert_eq!((img.width(), img.height()), (64, 64));
}

#[test]
fn recalc_normals_is_a_flag() {
    let out = out_path("recalc.png");
    assert_success(&run(stl_thumb()
        .arg("--recalc-normals")
        .arg("test_data/cube.stl")
        .arg(&out)));
    assert!(model_pixels(&image::open(&out).unwrap()) > 0);
}

#[test]
fn visible_is_a_flag() {
    // A missing model fails before any window is opened, so this only checks argument parsing:
    // clap exits with status 2 on a usage error, the application exits with 1.
    let out = run(stl_thumb().args(["-x", "test_data/missing.stl", "out.png"]));
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn missing_model_is_an_error_not_a_panic() {
    let out = run(stl_thumb().args([
        "test_data/missing.stl",
        &out_path("missing.png").to_string_lossy(),
    ]));
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Application error"));
}

#[cfg(target_os = "linux")]
#[test]
fn no_display_server_fails_cleanly_or_uses_osmesa() {
    let out = out_path("nodisplay.png");
    let result = run(stl_thumb()
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .arg("test_data/cube.stl")
        .arg(&out));
    match result.status.code() {
        // osmesa is installed and rendered the image
        Some(0) => assert!(model_pixels(&image::open(&out).unwrap()) > 0),
        // osmesa is unavailable: a clear error, not a panic (101)
        Some(1) => assert!(String::from_utf8_lossy(&result.stderr).contains("xvfb-run")),
        other => panic!(
            "unexpected exit {other:?}:\n{}",
            String::from_utf8_lossy(&result.stderr)
        ),
    }
}
