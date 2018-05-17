extern crate clap;

pub struct Config {
    pub stl_filename: String,
    pub img_filename: String,
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
            .get_matches();

        let stl_filename = matches.value_of("STL_FILE").unwrap().to_string();
        let img_filename = matches.value_of("IMG_FILE").unwrap().to_string();

        Config {
            stl_filename,
            img_filename,
        }
    }
}


