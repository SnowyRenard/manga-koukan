use std::{
    fs::File,
    io::{Cursor, Write},
    path::PathBuf,
};

use image::EncodableLayout;
use tar::{Builder, Header};

use super::Archive;

pub struct Tar {
    archive: Builder<File>,
}

impl Tar {
    pub(crate) fn new(file_path: &PathBuf) -> Self {
        let file = File::create(file_path).unwrap();
        let archive = Builder::new(file);

        Self { archive }
    }
    pub(crate) fn finish(&mut self) {
        self.archive.finish().unwrap();
    }
}

impl Archive for Tar {
    fn write_data(&mut self, data: Cursor<Vec<u8>>, file_name: &str) {
        // Create header
        let mut header = Header::new_gnu();
        header.set_mode(0o755); // Write access in u32 for octal

        let mut entry = self.archive.append_writer(&mut header, file_name).unwrap();
        entry.write(data.into_inner().as_bytes()).unwrap();
        entry.finish().unwrap();
    }
    fn write_file(&mut self, file: &mut File, file_name: &str) {
        self.archive.append_file(file_name, file).unwrap();
    }
}
