use core::panic;
use std::path::Path;

use clap::Parser;

use image::ImageFormat;
use log::{error, warn};
use manga_koukan::{
    config::{ArchiveFormat, Config},
    run,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
///Mangabuilder is a simple app that converts a directory to .cbt
pub struct Cli {
    ///The input path for the file or directory
    pub input: String,

    ///The output directory
    #[arg(short, long, default_value_t = String::from("./"))]
    pub output: String,

    ///The archive format for the final output
    #[arg(short, long, default_value_t = String::from("cbz"))]
    pub archive: String,

    ///The image format for the final images
    #[arg(short, long, default_value = None)]
    pub format: Option<String>,

    ///The resolution you want the image to be converted to
    #[arg(short, long, default_value = None)]
    pub resolution: Option<String>,
    // For quality settings don't forget codecs like bmp which have more contorl over color so could be ideal for gray images.
}

fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Warn)
        .parse_default_env()
        .init();

    let cli = Cli::parse();

    // Get input and output directories
    let input = Path::new(&cli.input).to_path_buf();
    let output = input.with_extension(&cli.archive);

    // Get the appropiate image format for conversion if needed
    let image_format = match cli.format {
        Some(f) => Some(ImageFormat::from_extension(f).unwrap()),
        None => None,
    };

    // Select the appropriate file format for saving
    let archive_format = match cli.archive.as_ref() {
        "cbz" => ArchiveFormat::CBZ,
        "cbt" => ArchiveFormat::CBT,
        f => {
            error!("Couldn't recognize format {}", f);
            panic!();
        }
    };

    let resolution = match &cli.resolution {
        Some(r) => {
            warn!("The resizing system is still incomplete and should be patched to make sure the best resolution based on user settings should be applied");
            let res: Vec<_> = r.split("x").collect();
            let width = res[0].parse().unwrap();
            let height = res[1].parse().unwrap();

            let result;

            // Check if the resolution is valid
            if width != 0 && height != 0 {
                result = Some([width, height]);
            } else {
                warn!("Width and height are zero, ignoring");
                result = None;
            }

            result
        }
        None => None,
    };

    let config = Config {
        input,
        output,

        image_format,
        archive_format,

        resolution,

        remove_margine: true,
        split_pages: true,
    };

    run(&config);
}
