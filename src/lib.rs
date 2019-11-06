use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process;

use image::{ImageBuffer, RgbImage};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_image_dimensions() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .to_rgb();
        let dims = img.dimensions();
        let opened_image = OpenImage { img, dims };
        let raw_data = opened_image.img.into_vec();
        let raw_buffer: std::vec::Vec<_> = raw_data.chunks_exact(3).collect();
        println!("{:?}", raw_buffer);

        assert_eq!((1024, 694), opened_image.dims);
    }

    #[test]
    fn print_rgb_first_row() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .to_rgb();
        let dims = img.dimensions();
        let opened_image = OpenImage { img, dims };

        for i in 0..dims.1 {
            println!("{:?}", opened_image.img.get_pixel(0, i));
        }
    }

    #[test]
    fn check_energy_calc() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .to_rgb();
        let dims = img.dimensions();
        let opened_image = OpenImage { img, dims };

        assert_eq!(0, opened_image.pixel_energy((10, 10)));
    }
}

#[derive(Debug)]
pub struct Config {
    pub img_path: String,
    pub reduce_by: u32,
}

pub struct OpenImage {
    pub img: RgbImage,
    pub dims: (u32, u32), //(width, height)
}

impl OpenImage {
    pub fn pixel_energy(&self, pos: (u32, u32)) -> usize {
        println!(
            "The pixel RGB Vals are {:?}",
            self.img.get_pixel(pos.0, pos.1)
        );
        match pos {
            (0, 0) => println!("Top Left Corner"),
            (0, y) => println!("Top Border"),
            (x, 0) => println!("Left Border"),
            (x, y) => {}
        }

        0
    }
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
