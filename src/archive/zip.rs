use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use image::EncodableLayout;
use zip::{
    write::{FileOptions, SimpleFileOptions},
    ZipWriter,
};

use super::Archive;

pub(crate) struct Zip {
    archive: ZipWriter<File>,
    options: FileOptions<'static, ()>,
}

impl Zip {
    pub(crate) fn new(file_path: &PathBuf) -> Self {
        let file = File::create(file_path).unwrap();
        let archive = ZipWriter::new(file);

        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::STORE)
            .unix_permissions(0o755);

        Self { archive, options }
    }
    pub(crate) fn finish(self) {
        self.archive.finish().unwrap();
    }
}

impl Archive for Zip {
    fn write_data(&mut self, data: std::io::Cursor<Vec<u8>>, file_name: &str) {
        self.archive.start_file(file_name, self.options).unwrap();
        self.archive.write(data.into_inner().as_bytes()).unwrap();
    }
    fn write_file(&mut self, file: &mut std::fs::File, file_name: &str) {
        self.archive.start_file(file_name, self.options).unwrap();
        let mut contents: Vec<u8> = Vec::new();
        file.read_to_end(&mut contents).unwrap();
        self.archive.write_all(&contents).unwrap();
    }
}
