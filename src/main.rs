#[macro_use]
extern crate log;
extern crate stderrlog;

extern crate stl_thumb;

use std::process;
use std::io;
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
    match config.img_filename {
        Some(ref name) => info!("Thumbnail File: {}\n", &name),
        None => info!("Output: stdout\n"),
    };

    match stl_thumb::run(&config) {
        Ok(img) => {
            if !config.visible {
                // Output image
                // ============

                // Write to stdout if user did not specify a file
                let mut output: Box<io::Write> = match config.img_filename {
                    Some(ref x) => {
                        Box::new(std::fs::File::create(&x).unwrap())
                    },
                    None => Box::new(io::stdout()),
                };
                img.write_to(&mut output, config.format.to_owned())
                    .expect("Error saving image");
            }
        },
        Err(e) => {
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
