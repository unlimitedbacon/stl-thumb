#[macro_use]
extern crate log;
extern crate stderrlog;

extern crate stl_thumb;

use std::env;
use std::process;
use stl_thumb::config::Config;

fn main() {
    // Workaround on Linux for issues with Mesa 18.3
    #[cfg(target_os = "linux")]
    env::set_var("MESA_GL_VERSION_OVERRIDE", "3.2");

    let config = Config::new();

    stderrlog::new()
        .module(module_path!())
        //.quiet(config.quiet)
        .verbosity(config.verbosity)
        //.timestamp(config.timestamp)
        .init()
        .unwrap();

    info!("STL File: {}", config.stl_filename);
    match config.img_filename {
        Some(ref name) => info!("Thumbnail File: {}\n", &name),
        None => info!("Output: stdout\n"),
    };

    if let Err(e) = stl_thumb::run(&config) {
        error!("Application error: {}", e);
        process::exit(1);
    }
}

// Notes
// =====
//
// Linux Thumbnails
// ----------------
// https://tecnocode.co.uk/2013/10/21/writing-a-gnome-thumbnailer/
// https://wiki.archlinux.org/index.php/XDG_MIME_Applications#Shared_MIME_database
// https://developer.gnome.org/integration-guide/stable/thumbnailer.html.en (outdated)
//
// Window Thumbnails
// -----------------
// https://code.msdn.microsoft.com/windowsapps/CppShellExtThumbnailHandler-32399b35
// https://github.com/Arlorean/Voxels
//
// Helpful Examples
// ----------------
// https://github.com/bwasty/gltf-viewer
//
// OpenGL
// ------
// https://glium-doc.github.io/#/
// http://www.opengl-tutorial.org/beginners-tutorials/tutorial-3-matrices/
