use std::{fs::File, io::Cursor, path::PathBuf, sync::Mutex};

use image::{DynamicImage, GenericImage, GenericImageView, ImageReader};

use crate::config::Config;

/// A global trait to implement all archives
pub trait Archive: Send {
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
    fn finish(self);
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
    if let Some(margin) = config.remove_margin {
        process_margin(&mut image, margin);
    }

    // Resize the image to the final resolution
    if let Some([mut w, mut h]) = config.resolution {
        if w != 0 || h != 0 {
            if w == 0 {
                w = (image.width() as f32 * (h as f32 / image.height() as f32)) as u32;
            }
            if h == 0 {
                h = (image.height() as f32 * (w as f32 / image.width() as f32)) as u32;
            }
            image = image.resize(w, h, image::imageops::FilterType::Lanczos3);
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

fn process_margin(image: &mut DynamicImage, margin: f32) {
    if image.width() > image.height() {
        let mut left_margin = image.width();
        let mut right_margin = 0;

        let margin_color = &image.get_pixel(image.width() - 1, 0);

        image
            .to_rgba8()
            .enumerate_pixels()
            .for_each(|(x, _, pixel)| {
                // Left
                if x < left_margin && pixel != margin_color {
                    left_margin = x;
                }
                // Right
                if x > right_margin && pixel != margin_color {
                    right_margin = x;
                }
            });

        let offset = left_margin;
        let width = right_margin - left_margin;

        if margin <= offset as f32 / image.width() as f32 {
            *image = image
                .sub_image(offset, 0, width, image.height())
                .to_image()
                .into();
        }
    }
}

fn split_pages(
    image: &mut DynamicImage,
    archive: &Mutex<impl Archive>,
    name: &str,
    format: image::ImageFormat,
) {
    let image_a = image
        .sub_image(0, 0, image.width() / 2, image.height())
        .to_image();
    let image_b = image
        .sub_image(
            (image.width() / 2) + 1,
            0,
            image.width() / 2,
            image.height(),
        )
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
