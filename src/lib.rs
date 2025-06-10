use std::{fs::File, path::PathBuf};

use archive::{tar::Tar, zip::Zip, Archive};
use rayon::prelude::*;

use config::{ArchiveFormat, Config};

mod archive;
pub mod config;

pub fn run(config: &Config) {
    // Get all the files recursively
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&config.input)
        .unwrap()
        .filter_map(|res| res.map(|entry| entry.path()).ok())
        // Check if the entry is a valid image.
        .filter(|entry| image::ImageFormat::from_path(&entry).is_ok())
        .collect();

    // entries.sort();
    entries.sort_by(|a, b| path_to_num(a).cmp(&path_to_num(b)));

    // Create archive
    match config.archive_format {
        ArchiveFormat::CBZ => {
            let mut archive = Zip::new(&config.output);
            compress(config, &mut archive, &entries);
            archive.finish();
        }
        ArchiveFormat::CBT => {
            let mut archive = Tar::new(&config.output);
            compress(config, &mut archive, &entries);
            archive.finish();
        }
    };
}
fn path_to_num(path: &PathBuf) -> usize {
    let mut string = path.to_string_lossy().into_owned();

    string.retain(|c| "0123456789".contains(c));

    string.parse().unwrap()
}

fn compress(config: &Config, archive: &mut impl Archive, entries: &Vec<PathBuf>) {
    if let Some(format) = config.image_format {
        // Convert all the files
        let mut pages = entries
            .par_iter()
            .flat_map(|e| archive::convert_page(config, e))
            .collect::<Vec<_>>();

        pages.sort();

        pages.iter().enumerate().for_each(|(index, page)| {
            archive.write_image(
                &page.image,
                format,
                &format!("{}.{}", index, format.extensions_str()[0]),
            )
        });

        // Generate cover image
        archive.write_image(
            &archive::convert_page(config, &entries[0])[0].image,
            format,
            "cover",
        );
    } else {
        entries.iter().for_each(|entry| {
            archive.write_file(
                &mut File::open(entry).unwrap(),
                entry.file_name().unwrap().to_str().unwrap(),
            )
        });
    }
}
