extern crate clap;

use image::ImageOutputFormat;
use std::f32;
use std::path::Path;

#[derive(Clone)]
pub struct Material {
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
}

#[derive(Clone)]
pub struct Config {
    pub stl_filename: String,
    pub img_filename: Option<String>,
    pub format: ImageOutputFormat,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
    pub verbosity: usize,
    pub material: Material,
    pub background: (f32, f32, f32, f32),
}

impl Default for Config {
    fn default() -> Self {
        Config {
            stl_filename: "".to_string(),
            img_filename: None,
            format: ImageOutputFormat::PNG,
            width: 1024,
            height: 768,
            visible: false,
            verbosity: 0,
            material: Material {
                ambient: [0.0, 0.0, 0.4],
                diffuse: [0.0, 0.5, 1.0],
                specular: [1.0, 1.0, 1.0],
            },
            background: (1.0, 1.0, 1.0, 0.0),
        }
    }
}

impl Config {
    pub fn new() -> Config {
        // Define command line arguments
        let matches = clap::App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .arg(
                clap::Arg::with_name("STL_FILE")
                    .help("STL file")
                    .required(true)
                    .index(1),
            )
            .arg(
                clap::Arg::with_name("IMG_FILE")
                    .help("Thumbnail image file. If this is omitted, the image data will be dumped to stdout.")
                    .index(2),
            )
            .arg(
                clap::Arg::with_name("format")
                    .help("The format of the image file. If not specified it will be determined from the file extension, or default to PNG if there is no extension. Supported formats: PNG, JPEG, GIF, ICO, BMP")
                    .short("f")
                    .long("format")
                    .takes_value(true)
            )
            .arg(
                clap::Arg::with_name("size")
                    .help("Size of thumbnail (square)")
                    .short("s")
                    .long("size")
                    .takes_value(true)
                    .required(false)
            )
            .arg(
                clap::Arg::with_name("visible")
                    .help("Display the thumbnail in a window instead of saving a file")
                    .short("x")
                    .required(false)
            )
            .arg(
                clap::Arg::with_name("verbosity")
                    .short("v")
                    .multiple(true)
                    .help("Increase message verbosity")
            )
            .arg(
                clap::Arg::with_name("material")
                    .help("Colors for rendering the mesh using the Phong reflection model. Requires 3 colors as rgb hex values: ambient, diffuse, and specular. Defaults to blue.")
                    .short("m")
                    .long("material")
                    .value_names(&["ambient","diffuse","specular"])
            )
            .arg(
                clap::Arg::with_name("background")
                    .help("The background color with transparency (rgba). Default is ffffff00.")
                    .short("b")
                    .long("background")
                    .takes_value(true)
                    .required(false)
            )
            .get_matches();

        let mut c = Config {
            ..Default::default()
        };

        c.stl_filename = matches.value_of("STL_FILE").unwrap().to_string();
        matches
            .value_of("IMG_FILE")
            .map(|x| c.img_filename = Some(x.to_string()));
        match matches.value_of("format") {
            Some(x) => c.format = match_format(x),
            None => match &c.img_filename {
                Some(x) => match Path::new(x).extension() {
                    Some(ext) => c.format = match_format(ext.to_str().unwrap()),
                    _ => (),
                },
                _ => (),
            },
        };
        matches
            .value_of("size")
            .map(|x| c.width = x.parse::<u32>().expect("Invalid size"));
        matches
            .value_of("size")
            .map(|x| c.height = x.parse::<u32>().expect("Invalid size"));
        c.visible = matches.is_present("visible");
        c.verbosity = matches.occurrences_of("verbosity") as usize;
        match matches.values_of("material") {
            Some(mut x) => {
                c.material = Material {
                    ambient: html_to_rgb(x.next().unwrap()),
                    diffuse: html_to_rgb(x.next().unwrap()),
                    specular: html_to_rgb(x.next().unwrap()),
                }
            }
            _ => (),
        };
        matches
            .value_of("background")
            .map(|x| c.background = html_to_rgba(x));

        c
    }
}

fn match_format(ext: &str) -> ImageOutputFormat {
    match ext.to_lowercase().as_ref() {
        "png" => ImageOutputFormat::PNG,
        "jpeg" => ImageOutputFormat::JPEG(95),
        "jpg" => ImageOutputFormat::JPEG(95),
        "gif" => ImageOutputFormat::GIF,
        "ico" => ImageOutputFormat::ICO,
        "bmp" => ImageOutputFormat::BMP,
        _ => {
            warn!("Unsupported image format. Using PNG instead.");
            Config {
                ..Default::default()
            }
            .format
        }
    }
}

fn html_to_rgb(color: &str) -> [f32; 3] {
    let r: f32 = u8::from_str_radix(&color[0..2], 16).expect("Invalid color") as f32 / 255.0;
    let g: f32 = u8::from_str_radix(&color[2..4], 16).expect("Invalid color") as f32 / 255.0;
    let b: f32 = u8::from_str_radix(&color[4..6], 16).expect("Invalid color") as f32 / 255.0;
    [r, g, b]
}

fn html_to_rgba(color: &str) -> (f32, f32, f32, f32) {
    let r: f32 = u8::from_str_radix(&color[0..2], 16).expect("Invalid color") as f32 / 255.0;
    let g: f32 = u8::from_str_radix(&color[2..4], 16).expect("Invalid color") as f32 / 255.0;
    let b: f32 = u8::from_str_radix(&color[4..6], 16).expect("Invalid color") as f32 / 255.0;
    let a: f32 = u8::from_str_radix(&color[6..8], 16).expect("Invalid color") as f32 / 255.0;
    (r, g, b, a)
}
