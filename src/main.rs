use clap::Parser;

use manga_builder::run;

fn main() {
    println!("Hello, world!");

    let config = manga_builder::config::Config::parse();

    run(&config);
}
