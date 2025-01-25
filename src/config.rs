use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
///Mangabuilder is a simple app that converts a directory to .cbz
pub struct Config {
    ///The input path for the file or directory
    #[arg(short, long)]
    pub input: String,

    ///The output path for the final document
    #[arg(short, long)]
    pub output: String,
}
