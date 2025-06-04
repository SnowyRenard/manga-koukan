use core::panic;
use std::path::PathBuf;

use clap::ValueEnum;
use image::ImageFormat;

pub struct Config {
    pub input: PathBuf,
    pub output: PathBuf,

    pub image_format: Option<ImageFormat>,
    pub archive_format: ArchiveFormat,

    pub resolution: Option<[u32; 2]>,

    pub remove_margin: Option<f32>,
    pub split_pages: bool,
}

#[derive(Clone, ValueEnum)]
pub enum ArchiveFormat {
    CBZ,
    CBT,
}

impl<'a> Into<&'a str> for ArchiveFormat {
    fn into(self) -> &'a str {
        match self {
            ArchiveFormat::CBZ => "cbz",
            ArchiveFormat::CBT => "cbz",
        }
    }
}
impl From<&str> for ArchiveFormat {
    fn from(value: &str) -> Self {
        match value {
            "cbz" => ArchiveFormat::CBZ,
            "cbt" => ArchiveFormat::CBT,
            _ => panic!("No Recoginzed format"),
        }
    }
}
