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
struct Cli {
    ///The input path for the file or directory
    input: String,

    ///The output directory
    #[arg(short, long, default_value_t = String::from("./"))]
    output: String,

    ///The archive format for the final output
    #[arg(short, long, default_value_t = String::from("cbz"))]
    archive: String,

    ///The image format for the final images
    #[arg(short, long, default_value = None)]
    format: Option<String>,

    ///The resolution you want the image to be converted to
    #[arg(short, long, default_value = None)]
    resolution: Option<String>,

    ///Try to remove the margin from all pages
    #[arg(long, default_value_t = true)]
    remove_margins: bool,

    ///Split the pages in half if they are wider than taller
    #[arg(long, default_value_t = true)]
    split_pages: bool,
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

    // Get the appropriate image format for conversion if needed
    let image_format = cli.format.map(|f| ImageFormat::from_extension(f).unwrap());

    // Select the appropriate file format for saving
    let archive_format = match cli.archive.as_ref() {
        "cbz" => ArchiveFormat::CBZ,
        "cbt" => ArchiveFormat::CBT,
        f => {
            error!("Couldn't recognize format {}", f);
            panic!();
        }
    };

    let resolution = cli.resolution.map(|r|{
        warn!("The resizing system is still incomplete and should be patched to make sure the best resolution based on user settings should be applied");
        let res: Vec<_> = r.split("x").collect();
        let width = res[0].parse().unwrap();
        let height = res[1].parse().unwrap();

        [width, height]
    });

    let config = Config {
        input,
        output,

        image_format,
        archive_format,

        resolution,

        remove_margine: cli.remove_margins,
        split_pages: cli.split_pages,
    };

    run(&config);
}
