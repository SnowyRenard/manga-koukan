use std::{cmp::Ordering, fs::File, io::Cursor, path::PathBuf};

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

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Side {
    None,
    Right,
    Left,
}
#[derive(PartialEq)]
pub struct Page {
    pub image: DynamicImage,
    pub(crate) index: usize,
    pub(crate) side: Side,
}
impl From<&PathBuf> for Page {
    fn from(path: &PathBuf) -> Self {
        let mut index = path.to_string_lossy().into_owned();
        index.retain(|c| "0123456789".contains(c));

        Self {
            image: ImageReader::open(path).unwrap().decode().unwrap(),
            index: index.parse().unwrap_or_default(),
            side: Side::None,
        }
    }
}
impl Eq for Page {}
impl Ord for Page {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.index.cmp(&other.index) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.side.cmp(&other.side),
            Ordering::Greater => Ordering::Greater,
        }
    }
}
impl PartialOrd for Page {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index).map(|cmp| match cmp {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self
                .side
                .partial_cmp(&other.side)
                .unwrap_or(Ordering::Equal),
            Ordering::Greater => Ordering::Greater,
        })
    }
}

pub(crate) mod tar;
pub(crate) mod zip;

pub(crate) fn convert_page(config: &Config, entry: &PathBuf) -> Vec<Page> {
    let mut pages = split_pages(entry.into(), config);

    // Remove the margins on the image
    if let Some(margin) = config.remove_margin {
        pages
            .iter_mut()
            .for_each(|page| process_margin(&mut page.image, margin));
    }

    pages
        .iter_mut()
        .for_each(|page| resize(&mut page.image, config));

    pages
}

fn process_margin(image: &mut DynamicImage, margin: f32) {
    let mut left_margin = vec![image.width() / 2; image.height() as usize];
    let mut right_margin = vec![image.width() / 2; image.height() as usize];

    let margin_color = &image.get_pixel(image.width() - 1, 0);

    image
        .to_rgba8()
        .enumerate_pixels()
        .for_each(|(x, y, pixel)| {
            // Left
            if x < left_margin[y as usize] && pixel != margin_color {
                left_margin[y as usize] = x;
            }
            // Right
            if x > right_margin[y as usize] && pixel != margin_color {
                right_margin[y as usize] = x;
            }
        });

    let left_margin = vec_most_occur(&left_margin);
    let right_margin = vec_most_occur(&right_margin);

    let offset = left_margin;
    let width = right_margin - left_margin;

    if margin < (width - image.width()) as f32 / image.width() as f32 {
        *image = image
            .sub_image(offset, 0, width, image.height())
            .to_image()
            .into();
    }
}

fn vec_most_occur<T: Eq + std::hash::Hash + Copy>(data: &[T]) -> T {
    let mut counter = std::collections::HashMap::new();
    for num in data.iter() {
        *counter.entry(num).or_insert(0) += 1;
    }

    *counter
        .iter()
        .reduce(|acc, e| if acc.1 > e.1 { e } else { acc })
        .unwrap()
        .0
        .to_owned()
}

fn resize(image: &mut DynamicImage, config: &Config) {
    if let Some([mut w, mut h]) = config.resolution {
        if w != 0 || h != 0 {
            if w == 0 {
                w = (image.width() as f32 * (h as f32 / image.height() as f32)) as u32;
            }
            if h == 0 {
                h = (image.height() as f32 * (w as f32 / image.width() as f32)) as u32;
            }
            *image = image.resize(w, h, image::imageops::FilterType::Lanczos3);
        }
    }
}

fn split_pages(page: Page, config: &Config) -> Vec<Page> {
    if config.split_pages && page.image.width() > page.image.height() {
        let mut image = page.image;
        let page_a = Page {
            image: image
                .sub_image(0, 0, image.width() / 2, image.height())
                .to_image()
                .into(),
            index: page.index,
            side: Side::Left,
        };
        let page_b = Page {
            image: image
                .sub_image(image.width() / 2, 0, image.width() / 2, image.height())
                .to_image()
                .into(),
            index: page.index,
            side: Side::Right,
        };

        vec![page_a, page_b]
    } else {
        vec![page]
    }
}
