use std::{fs::File, io::Cursor, path::PathBuf, sync::Mutex};

use image::{DynamicImage, GenericImage, GenericImageView, ImageReader};

use crate::config::Config;

/// A global trait to implement all archives
pub(crate) trait Archive: Sync + Send {
    /// Write arbitrary data to archive
    fn write_data(&mut self, data: Cursor<Vec<u8>>, file_name: &str);
    /// Write a pre existing file to archive
    fn write_file(&mut self, file: &mut File, file_name: &str);
    /// Write a [`image::DynamicImage`] to archive
    fn write_image(&mut self, image: &DynamicImage, format: image::ImageFormat, file_name: &str) {
        let mut data = Cursor::new(Vec::new());

        image.write_to(&mut data, format).unwrap();
        self.write_data(data, file_name);
    }
}

pub(crate) mod tar;
pub(crate) mod zip;

pub(crate) fn create_page(config: &Config, archive: &Mutex<impl Archive>, entry: &PathBuf) {
    match config.image_format {
        Some(_) => convert_page(config, archive, entry),
        None => load_page(archive, entry),
    }
}

pub(crate) fn create_page_with_name(
    config: &Config,
    archive: &Mutex<impl Archive>,
    entry: &PathBuf,
    name: &str,
) {
    match config.image_format {
        Some(_) => convert_page_with_name(config, archive, entry, name),
        None => load_page_with_name(archive, entry, name),
    }
}

fn load_page(archive: &Mutex<impl Archive>, entry: &PathBuf) {
    load_page_with_name(archive, entry, entry.to_str().unwrap())
}

fn load_page_with_name(archive: &Mutex<impl Archive>, entry: &PathBuf, name: &str) {
    // Write file from dir to archive
    archive
        .lock()
        .unwrap()
        .write_file(&mut File::open(entry.as_path()).unwrap(), name);
}

fn convert_page(config: &Config, archive: &Mutex<impl Archive>, entry: &PathBuf) {
    let name = entry.file_stem().unwrap().to_str().unwrap();
    create_page_with_name(config, archive, entry, &name)
}

fn convert_page_with_name(
    config: &Config,
    archive: &Mutex<impl Archive>,
    entry: &PathBuf,
    name: &str,
) {
    // Var prep
    let mut image = ImageReader::open(entry).unwrap().decode().unwrap();
    let format = config.image_format.unwrap();

    // Remove the margins on the image
    if config.remove_margine {
        process_margin(&mut image);
    }

    // Resize the image to the final resolution
    if let Some(r) = config.resolution {
        if r[0] > 0 && r[1] > 0 {
            image = image.resize(r[0], r[1], image::imageops::FilterType::Lanczos3);
        }
    }

    // Change the extension of the file to the new image format

    if config.split_pages && image.width() > image.height() {
        split_pages(&mut image, archive, name, format);
    } else {
        // Write the data to the archive
        let file_name = format!("{}.{}", name, format.extensions_str()[0]);
        archive
            .lock()
            .unwrap()
            .write_image(&image, format, &file_name);
    }
}

fn process_margin(image: &mut DynamicImage) {
    if image.width() > image.height() {
        let mut offset = 0;

        let margin_color = &image.get_pixel(image.width() - 1, 0);

        image
            .to_rgba8()
            .enumerate_pixels()
            .for_each(|(x, _, pixel)| {
                // Right
                if x > offset && pixel != margin_color {
                    offset = x;
                }
            });

        let margin = image.width() - offset;

        *image = image
            .sub_image(margin, 0, image.width() - margin * 2, image.height())
            .to_image()
            .into();
    }
}

fn split_pages(
    image: &mut DynamicImage,
    archive: &Mutex<impl Archive>,
    name: &str,
    format: image::ImageFormat,
) {
    let image_a = image
        .sub_image(0, 0, image.width() / 2 - 1, image.height())
        .to_image();
    let image_b = image
        .sub_image(image.width() / 2, 0, image.width() / 2, image.height())
        .to_image();

    archive.lock().unwrap().write_image(
        &image_a.into(),
        format,
        &format!("{}a.{}", name, format.extensions_str()[0]),
    );
    archive.lock().unwrap().write_image(
        &image_b.into(),
        format,
        &format!("{}b.{}", name, format.extensions_str()[0]),
    );
}
