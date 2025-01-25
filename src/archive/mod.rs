use std::{fs::File, io::Cursor, path::PathBuf, sync::Mutex};

use image::{GenericImageView, ImageReader, Pixel, Rgb};

use crate::config::Config;

/// A global trait to implement all archives
pub(crate) trait Archive: Sync + Send {
    /// Write arbitrary data to archive
    fn write_data(&mut self, data: Cursor<Vec<u8>>, file_name: &str);
    /// Write a pre exisiting file to archive
    fn write_file(&mut self, file: &mut File, file_name: &str);
}

pub(crate) mod tar;
pub(crate) mod zip;

pub(crate) fn create_page<A: Archive>(config: &Config, archive: &Mutex<A>, entry: &PathBuf) {
    match config.image_format {
        Some(_) => convert_page(config, archive, entry),
        None => load_page(config, archive, entry),
    }
}

pub(crate) fn create_page_with_name<A: Archive>(
    config: &Config,
    archive: &Mutex<A>,
    entry: &PathBuf,
    name: &str,
) {
    match config.image_format {
        Some(_) => convert_page_with_name(config, archive, entry, name),
        None => load_page_with_name(config, archive, entry, name),
    }
}

fn load_page<A: Archive>(config: &Config, archive: &Mutex<A>, entry: &PathBuf) {
    load_page_with_name(config, archive, entry, entry.to_str().unwrap())
}

fn load_page_with_name<A: Archive>(
    _config: &Config,
    archive: &Mutex<A>,
    entry: &PathBuf,
    name: &str,
) {
    // Write file from dir to archive
    archive
        .lock()
        .unwrap()
        .write_file(&mut File::open(entry.as_path()).unwrap(), name);
}

fn convert_page<A: Archive>(config: &Config, archive: &Mutex<A>, entry: &PathBuf) {
    let name = entry.file_stem().unwrap().to_str().unwrap();
    create_page_with_name(config, archive, entry, &name)
}

fn convert_page_with_name<A: Archive>(
    config: &Config,
    archive: &Mutex<A>,
    entry: &PathBuf,
    name: &str,
) {
    // Var prep
    let mut image = ImageReader::open(entry).unwrap().decode().unwrap();
    let format = config.image_format.unwrap();
    let mut data = Cursor::new(Vec::new());

    // Remove the margins on the image
    if config.remove_margine {
        #[allow(unused)]
        if image.width() > image.height() {
            // Setup
            let mut left_margin = 0;
            let mut right_margin = image.width() - 1;

            let left_margin_color = &image.get_pixel(left_margin, 0);
            let right_margin_color = &image.get_pixel(right_margin, 0);

            // Left margin

            if (left_margin_color.to_rgb() != Rgb::from([0, 0, 0])) // Make sure the margin isn't white
                || (left_margin_color.to_rgb() != Rgb::from([255, 255, 255]))
            // Or black
            {
                for w in 0..image.width() {
                    for h in 0..image.height() {
                        break;
                    }
                }
            }
            //  Aproach
            //  first go from outside to inside and try to find the first pixel that is different
            //  move down until you have found a different pixel
            //  if a pixel is different then move outside one pixel and start from the top again
            //  if you reach the bottom then you have found the margin
        }
    }

    // Resize the image to the final resolution
    match &config.resolution {
        Some(r) => {
            image = image.resize(r[0], r[1], image::imageops::FilterType::Lanczos3);
        }
        None => (),
    }

    image.write_to(&mut data, format).unwrap();

    // Change the extension of the file to the new image format
    let file_name = format!("{}.{}", name, format.extensions_str()[0]);

    // Write the data to the archive
    archive.lock().unwrap().write_data(data, &file_name);
}
