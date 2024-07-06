use image::ImageFormat;
use std::f32;
use std::path::Path;

#[derive(Clone)]
pub struct Material {
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
}

#[derive(Clone)]
pub enum AAMethod {
    None,
    FXAA,
}

#[derive(Clone)]
pub struct Config {
    pub stl_filename: String,
    pub img_filename: String,
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
    pub verbosity: usize,
    pub material: Material,
    pub background: (f32, f32, f32, f32),
    pub aamethod: AAMethod,
    pub recalc_normals: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            stl_filename: "".to_string(),
            img_filename: "".to_string(),
            format: ImageFormat::Png,
            width: 1024,
            height: 768,
            visible: false,
            verbosity: 0,
            material: Material {
                ambient: [0.1552, 0.0086, 0.0266],
                diffuse: [0.5432, 0.0301, 0.0931],
                specular: [1.0, 1.0, 1.0],
            },
            background: (0.0, 0.0, 0.0, 0.0),
            aamethod: AAMethod::FXAA,
            recalc_normals: false,
        }
    }
}

impl Config {
    pub fn new() -> Config {
        // Define command line arguments
        let mut matches = clap::Command::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .arg(
                clap::Arg::new("STL_FILE")
                    .help("STL file. Use - to read from stdin instead of a file.")
                    .required(true)
                    .index(1),
            )
            .arg(
                clap::Arg::new("IMG_FILE")
                    .help("Thumbnail image file. Use - to write to stdout instead of a file.")
                    .required(true)
                    .index(2),
            )
            .arg(
                clap::Arg::new("format")
                    .help("The format of the image file. If not specified it will be determined from the file extension, or default to PNG if there is no extension. Supported formats: PNG, JPEG, GIF, ICO, BMP")
                    .short('f')
                    .long("format")
                    .action(clap::ArgAction::Set)
            )
            .arg(
                clap::Arg::new("size")
                    .help("Size of thumbnail (square)")
                    .short('s')
                    .long("size")
                    .action(clap::ArgAction::Set)
                    .required(false)
            )
            .arg(
                clap::Arg::new("visible")
                    .help("Display the thumbnail in a window instead of saving a file")
                    .short('x')
                    .required(false)
            )
            .arg(
                clap::Arg::new("verbosity")
                    .short('v')
                    .action(clap::ArgAction::Count)
                    .help("Increase message verbosity")
            )
            .arg(
                clap::Arg::new("material")
                    .help("Colors for rendering the mesh using the Phong reflection model. Requires 3 colors as rgb hex values: ambient, diffuse, and specular. Defaults to blue.")
                    .short('m')
                    .long("material")
                    .value_names(&["ambient","diffuse","specular"])
            )
            .arg(
                clap::Arg::new("background")
                    .help("The background color with transparency (rgba). Default is ffffff00.")
                    .short('b')
                    .long("background")
                    .action(clap::ArgAction::Set)
                    .required(false)
            )
            .arg(
                clap::Arg::new("aamethod")
                    .help("Anti-aliasing method. Default is FXAA, which is fast but may introduce artifacts.")
                    .short('a')
                    .long("antialiasing")
                    .value_parser(["none", "fxaa"]),
            )
            .arg(
                clap::Arg::new("recalc_normals")
                    .help("Force recalculation of face normals. Use when dealing with malformed STL files.")
                    .long("recalc-normals")
            )
            .get_matches();

        let mut c = Config {
            ..Default::default()
        };

        c.stl_filename = matches
            .remove_one::<String>("STL_FILE")
            .expect("STL_FILE not provided");
        c.img_filename = matches
            .remove_one::<String>("IMG_FILE")
            .expect("IMG_FILE not provided");
        match matches.get_one::<String>("format") {
            Some(x) => c.format = match_format(x),
            None => match Path::new(&c.img_filename).extension() {
                Some(ext) => c.format = match_format(ext.to_str().unwrap()),
                _ => (),
            },
        };
        matches
            .get_one::<String>("size")
            .map(|x| c.width = x.parse::<u32>().expect("Invalid size"));
        matches
            .get_one::<String>("size")
            .map(|x| c.height = x.parse::<u32>().expect("Invalid size"));
        c.visible = matches.contains_id("visible");
        c.verbosity = matches.get_count("verbosity") as usize;
        if let Some(materials) = matches.get_many::<String>("material") {
            let mut iter = materials.map(|m| html_to_rgb(m));
            c.material = Material {
                ambient: iter.next().unwrap_or([0.0, 0.0, 0.0]),
                diffuse: iter.next().unwrap_or([0.0, 0.0, 0.0]),
                specular: iter.next().unwrap_or([0.0, 0.0, 0.0]),
            };
        }
        matches
            .get_one::<String>("background")
            .map(|x| c.background = html_to_rgba(x));
        match matches.get_one::<String>("aamethod") {
            Some(x) => match x.as_str() {
                "none" => c.aamethod = AAMethod::None,
                "fxaa" => c.aamethod = AAMethod::FXAA,
                _ => unreachable!(),
            },
            None => (),
        };
        c.recalc_normals = matches.contains_id("recalc_normals");

        c
    }
}

fn match_format(ext: &str) -> ImageFormat {
    match ext.to_lowercase().as_str() {
        "png" => ImageFormat::Png,
        "jpeg" | "jpg" => ImageFormat::Jpeg,
        "gif" => ImageFormat::Gif,
        "ico" => ImageFormat::Ico,
        "bmp" => ImageFormat::Bmp,
        _ => {
            warn!("Unsupported image format. Using PNG instead.");
            ImageFormat::Png
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
