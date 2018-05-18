extern crate clap;

pub struct Config {
    pub stl_filename: String,
    pub img_filename: String,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
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
                    .help("Thumbnail image")
                    .required(true)
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
                    .short("v")
                    .required(false)
            )
            .get_matches();

        let stl_filename = matches.value_of("STL_FILE").unwrap().to_string();
        let img_filename = matches.value_of("IMG_FILE").unwrap().to_string();
        let width = matches.value_of("size").unwrap_or("1024");
        let height = matches.value_of("size").unwrap_or("768");
        let width = width.parse::<u32>()
            .expect("Invalid size");
        let height = height.parse::<u32>()
            .expect("Invalid size");
        let visible = matches.is_present("visible");

        Config {
            stl_filename,
            img_filename,
            width,
            height,
            visible,
        }
    }
}


