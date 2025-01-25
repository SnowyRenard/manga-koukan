use std::fs::File;

use config::Config;
use image::{DynamicImage, ImageReader};
use tar::Builder;

pub mod config;

struct Page {
    pub name: String,
    pub data: DynamicImage,
}
impl Page {
    fn new(dir: String) -> Self {
        let data = ImageReader::open(&dir).unwrap().decode().unwrap();
        Self { name: dir, data }
    }
}

pub fn run(config: &Config) -> () {
    println!("{:?}", config.input);
    let pages = read_dir(config);

    write_cbt(config, pages);
}

fn read_dir(config: &Config) -> Vec<Page> {
    let mut entries = std::fs::read_dir(&config.input)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .unwrap();

    entries.sort();

    let mut pages = vec![];
    for e in entries {
        let name = e.to_string_lossy().to_string();
        println!("{}", name);
        pages.push(Page::new(name));
    }

    pages
}

fn write_cbt(config: &Config, images: Vec<Page>) {
    let file = File::create(&config.output).unwrap();
    let mut a = Builder::new(file);

    for i in images {
        a.append_file(&i.name, &mut File::open(&i.name).unwrap())
            .unwrap();
    }

    a.finish().unwrap();
}
