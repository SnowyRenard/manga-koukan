use std::path::PathBuf;

use clap::Parser;
use manga_koukan::{
    config::{ArchiveFormat, Config},
    run,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    ///The input path for the file or directory
    input: PathBuf,

    ///The output directory
    #[arg(short, long)]
    output: Option<PathBuf>,

    ///The archive format for the final output
    #[arg(short, long, value_enum, default_value_t = ArchiveFormat::CBZ)]
    archive: ArchiveFormat,

    ///The image format for the final images
    #[arg(short, long)]
    format: Option<String>,

    ///The resolution you want the image to be converted to
    #[arg(short, long)]
    resolution: Option<String>,

    ///Try to remove the margin from all pages
    #[arg(long)]
    remove_margins: Option<f32>,

    ///Split the pages in half if they are wider than taller
    #[arg(long, default_value_t = false)]
    split_pages: bool,
}

fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Warn)
        .parse_default_env()
        .init();

    let cli = Cli::parse();

    // Get input and output directories
    let output = cli
        .output
        .unwrap_or(cli.input.clone())
        .with_extension(&Into::<&str>::into(cli.archive.clone()));

    // Get the appropriate image format for conversion if needed
    let image_format = cli
        .format
        .map(|f| image::ImageFormat::from_extension(f).unwrap());

    let resolution = cli.resolution.map(|r| {
        let res: Vec<_> = r.split("x").collect();
        let width = res[0].parse().unwrap();
        let height = res[1].parse().unwrap();

        [width, height]
    });

    let config = Config {
        input: cli.input,
        output,

        image_format,
        archive_format: cli.archive,

        resolution,

        remove_margin: cli.remove_margins,
        split_pages: cli.split_pages,
    };

    run(&config);
}
