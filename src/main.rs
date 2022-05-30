#[macro_use]
extern crate log;
extern crate stderrlog;

extern crate stl_thumb;

use std::process;
use stl_thumb::config::Config;

#[cfg(target_os = "linux")]
use std::env;

fn main() {
    // Workaround for issues with OpenGL 3.1 on Mesa 18.3
    #[cfg(target_os = "linux")]
    env::set_var("MESA_GL_VERSION_OVERRIDE", "2.1");

    let config = Config::new();

    stderrlog::new()
        .module(module_path!())
        //.quiet(config.quiet)
        .verbosity(config.verbosity)
        //.timestamp(config.timestamp)
        .init()
        .unwrap();

    info!("STL File: {}", config.stl_filename);
    info!("IMG File: {}", config.img_filename);

    if config.visible {
        if let Err(e) = stl_thumb::render_to_window(config) {
            error!("Application error: {}", e);
            process::exit(1);
        }
    } else {
        if let Err(e) = stl_thumb::render_to_file(&config) {
            error!("Application error: {}", e);
            process::exit(1);
        }
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
