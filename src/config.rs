extern crate clap;

pub struct Config {
    pub stl_filename: String,
    pub img_filename: Option<String>,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
    pub verbosity: usize,
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
                clap::Arg::with_name("size")
                    .help("Size of thumbnail (square)")
                    .short("s")
                    .long("size")
                    .takes_value(true)
                    .required(false)
            )
            .arg(
                clap::Arg::with_name("visible")
                    .help("Display the thumbnail in a window")
                    .short("x")
                    .required(false)
            )
            .arg(
                clap::Arg::with_name("verbosity")
                    .short("v")
                    .multiple(true)
                    .help("Increase message verbosity")
            )
            .get_matches();

        let stl_filename = matches.value_of("STL_FILE").unwrap().to_string();
        let img_filename = match matches.value_of("IMG_FILE") {
            Some(x) => Some(x.to_string()),
            None => None,
        };
        let width = matches.value_of("size").unwrap_or("1024");
        let height = matches.value_of("size").unwrap_or("768");
        let width = width.parse::<u32>()
            .expect("Invalid size");
        let height = height.parse::<u32>()
            .expect("Invalid size");
        let visible = matches.is_present("visible");
        let verbosity = matches.occurrences_of("verbosity") as usize;

        Config {
            stl_filename,
            img_filename,
            width,
            height,
            visible,
            verbosity,
        }
    }
}


