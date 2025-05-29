use std::{path::PathBuf, sync::Mutex};

use archive::{create_page, create_page_with_name, tar::Tar, zip::Zip, Archive};
use rayon::prelude::*;

use config::{ArchiveFormat, Config};

mod archive;
pub mod config;

pub fn run(config: &Config) {
    // Get all the files recursively
    let entries = std::fs::read_dir(&config.input)
        .unwrap()
        .filter_map(|res| res.map(|e| e.path()).ok())
        .collect();

    // Create archive
    match config.archive_format {
        ArchiveFormat::CBZ => {
            let archive = Mutex::new(Zip::new(&config.output));
            compress(config, &archive, &entries);
            archive.into_inner().unwrap().finish();
        }
        ArchiveFormat::CBT => {
            let archive = Mutex::new(Tar::new(&config.output));
            compress(config, &archive, &entries);
            archive.into_inner().unwrap().finish();
        }
    };
}

fn compress(config: &Config, archive: &Mutex<impl Archive>, entries: &Vec<PathBuf>) {
    // Convert all the files
    entries.par_iter().for_each(|e| {
        create_page(config, archive, e);
    });

    // Generate cover image
    create_page_with_name(config, archive, &entries[0], "cover");
}
