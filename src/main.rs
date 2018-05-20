extern crate stl_thumb;

use std::process;
use stl_thumb::config::Config;

fn main() {
    let config = Config::new();

    println!("STL File: {}", config.stl_filename);
    println!("Thumbnail File: {}", config.img_filename);

    if let Err(e) = stl_thumb::run(&config) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}

// Notes
// =====
// https://tecnocode.co.uk/2013/10/21/writing-a-gnome-thumbnailer/
// https://wiki.archlinux.org/index.php/XDG_MIME_Applications#Shared_MIME_database
// https://developer.gnome.org/integration-guide/stable/thumbnailer.html.en (outdated)
// https://github.com/bwasty/gltf-viewer
// https://code.msdn.microsoft.com/windowsapps/CppShellExtThumbnailHandler-32399b35
