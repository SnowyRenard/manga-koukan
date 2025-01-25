use std::path::PathBuf;

use image::ImageFormat;

pub struct Config {
    pub input: PathBuf,
    pub output: PathBuf,

    pub image_format: Option<ImageFormat>,
    pub archive_format: ArchiveFormat,

    pub resolution: Option<[u32; 2]>,

    pub remove_margine: bool,
    pub split_pages: bool,
}

pub enum ArchiveFormat {
    CBZ,
    CBT,
}
