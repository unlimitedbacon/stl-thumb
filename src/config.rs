use image::ImageFormat;
use std::f32;
use std::path::Path;
use ini::Ini;
use std::env;

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
    pub model_filename: String,
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
        let cfg = Config {
            model_filename: "".to_string(),
            img_filename: "".to_string(),
            format: ImageFormat::Png,
            width: 1024,
            height: 768,
            visible: false,
            verbosity: 0,
            material: Material {
                ambient: [0.00, 0.13, 0.26],
                diffuse: [0.38, 0.63, 1.00],
                specular: [1.00, 1.00, 1.00],
            },
            background: (0.0, 0.0, 0.0, 0.0),
            aamethod: AAMethod::FXAA,
            recalc_normals: false,
        };

        // at the moment unused, allow specification via cli.
        let override_config_file = "".to_string();
        Self::read_default_values_from_ini(cfg, override_config_file)

    }
}

impl Config {
    pub fn new() -> Config {
        // Define command line arguments
        let mut matches = clap::Command::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .arg(
                clap::Arg::new("MODEL_FILE")
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
                    .value_names(["ambient","diffuse","specular"])
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

        c.model_filename = matches
            .remove_one::<String>("MODEL_FILE")
            .expect("MODEL_FILE not provided");
        c.img_filename = matches
            .remove_one::<String>("IMG_FILE")
            .expect("IMG_FILE not provided");
        match matches.get_one::<String>("format") {
            Some(x) => c.format = match_format(x),
            None => {
                if let Some(ext) = Path::new(&c.img_filename).extension() {
                    c.format = match_format(ext.to_str().unwrap());
                }
            }
        };

        if let Some(x) = matches.get_one::<String>("size") {
            c.width = x.parse::<u32>().expect("Invalid size");
        }

        if let Some(x) = matches.get_one::<String>("size") {
            c.height = x.parse::<u32>().expect("Invalid size");
        }

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
        if let Some(x) = matches.get_one::<String>("background") {
            c.background = html_to_rgba(x);
        }
        if let Some(x) = matches.get_one::<String>("aamethod") {
            match x.as_str() {
                "none" => c.aamethod = AAMethod::None,
                "fxaa" => c.aamethod = AAMethod::FXAA,
                _ => unreachable!(),
            }
        }
        c.recalc_normals = matches.contains_id("recalc_normals");

        c
    }

    fn read_default_values_from_ini(mut hardcoded: Config, config_file_path_override: String) -> Config {

        let config_locations = [
            config_file_path_override,
            env::var("HOME").unwrap()+"/.config/stl-thumb/conf.ini",
            "/etc/stl-thumb/conf.ini".to_string(),
        ];

        let config = config_locations
            .into_iter()
            .filter(|path| Path::new(path).exists())
            .next();

        if config.is_none() {
            return hardcoded
        }

        let conf = Ini::load_from_file(config.unwrap()).unwrap();
        let colors = conf.section(Some("Colors")).unwrap();

        if let Some(diffuse) = colors.get("color") {
            hardcoded.material.diffuse = html_to_rgb(&*diffuse.replace("#", ""));
            hardcoded.material.ambient[0] = hardcoded.material.diffuse[0]*0.1;
            hardcoded.material.ambient[1] = hardcoded.material.diffuse[1]*0.1;
            hardcoded.material.ambient[2] = hardcoded.material.diffuse[2]*0.1;
        }

        if let Some(red) = colors.get("red") {
            let red_value = red.parse::<f32>().unwrap();
            hardcoded.material.diffuse[0] = red_value;
            hardcoded.material.ambient[0] = red_value * 0.1;
        }

        if let Some(green) = colors.get("green") {
            let green_value = green.parse::<f32>().unwrap();
            hardcoded.material.diffuse[1] = green_value;
            hardcoded.material.ambient[1] = green_value * 0.1;
        }

        if let Some(blue) = colors.get("blue") {
            let blue_value = blue.parse::<f32>().unwrap();
            hardcoded.material.diffuse[2] = blue_value;
            hardcoded.material.ambient[2] = blue_value * 0.1;
        }

        if let Some(bg) = colors.get("background") {
            hardcoded.background = html_to_rgba(&*bg.replace("#", ""));
        }

        let renderer = conf.section(Some("Renderer")).unwrap();

        if let Some(aamethod) = renderer.get("aamethod") {
            match aamethod {
                "none" => hardcoded.aamethod = AAMethod::None,
                "fxaa" => hardcoded.aamethod = AAMethod::FXAA,
                _ => {}
            }
        }

        if let Some(width) = colors.get("width") {
            hardcoded.width = width.parse().unwrap()
        }

        if let Some(height) = colors.get("height") {
            hardcoded.height = height.parse().unwrap()
        }

        if let Some(format) = colors.get("format") {
            match format {
                "png" => hardcoded.format = ImageFormat::Png,
                "jpeg" => hardcoded.format = ImageFormat::Jpeg,
                "gif" => hardcoded.format = ImageFormat::Gif,
                "webp" => hardcoded.format = ImageFormat::WebP,
                "pnm" => hardcoded.format = ImageFormat::Pnm,
                "tiff" => hardcoded.format = ImageFormat:: Tiff,
                "tga" => hardcoded.format = ImageFormat::Tga,
                "dds" => hardcoded.format = ImageFormat::Dds,
                "bmp" => hardcoded.format = ImageFormat:: Bmp,
                "ico" => hardcoded.format = ImageFormat::Ico,
                "hdr" => hardcoded.format = ImageFormat::Hdr,
                "openexr" => hardcoded.format = ImageFormat::OpenExr,
                "farbfeld" => hardcoded.format = ImageFormat::Farbfeld,
                "avif" => hardcoded.format = ImageFormat::Avif,
                "qoi" => hardcoded.format = ImageFormat::Qoi,
                _ => {}
            }
        }

        if let Some(recalc_normals) = colors.get("recalc_normals") {
            hardcoded.recalc_normals = recalc_normals.parse().unwrap()
        }

        hardcoded
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
