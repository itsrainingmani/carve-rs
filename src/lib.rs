use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process;

use image::{ImageBuffer, RgbImage};

#[derive(Debug)]
pub struct Config {
    pub img_path: String,
    pub reduce_by: u32,
}

pub struct OpenImage {
    pub img: RgbImage,
    pub dims: (u32, u32),
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        args.next();

        let img_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a filename"),
        };

        let reduce_by = match args.next() {
            Some(arg) => arg.parse().unwrap_or_else(|err| {
                eprintln!(
                    "Couldn't parse the provided argument as an integer: {}",
                    err
                );
                process::exit(1);
            }),
            None => return Err("Didn't get the percentage to seam carve the image by"),
        };

        Ok(Config {
            img_path,
            reduce_by,
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let img = image::open(Path::new(&config.img_path))?.to_rgb();
    let dims = img.dimensions();
    let opened_image = OpenImage { img, dims };
    Ok(())
}
   k