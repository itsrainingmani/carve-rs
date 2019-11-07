use std::error::Error;
use std::path::Path;
use std::process;

use image::RgbImage;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_image_dimensions() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .to_rgb();
        let dims = img.dimensions();
        let buffer = img.to_vec();
        let opened_image = OpenImage { img, dims, buffer };
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
        let buffer = img.to_vec();
        let opened_image = OpenImage { img, dims, buffer };

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
        let buffer = img.to_vec();
        let opened_image = OpenImage { img, dims, buffer };

        assert_eq!(33822, opened_image.pixel_energy((0, 0)));
    }

    #[test]
    fn adjacency_test_corner_pixels() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .to_rgb();
        let dims = img.dimensions();
        let buffer = img.to_vec();
        let opened_image = OpenImage { img, dims, buffer };

        let top_left_corner: (u32, u32) = (0, 0);
        let top_left_adjacency = AdjacentPixels {
            left: (1023, 0),
            right: (1, 0),
            up: (0, 693),
            down: (0, 1),
        };

        let top_right_corner: (u32, u32) = (1023, 0);
        let top_right_adjacency = AdjacentPixels {
            left: (1022, 0),
            right: (0, 0),
            up: (1023, 693),
            down: (1023, 1),
        };

        assert_eq!(
            vec![top_left_adjacency, top_right_adjacency],
            vec![
                opened_image.get_adjacent_pixels(top_left_corner),
                opened_image.get_adjacent_pixels(top_right_corner)
            ]
        );
    }
}

#[derive(Debug)]
pub struct Config {
    pub img_path: String,
    pub reduce_by: u32,
}

#[derive(Debug)]
pub struct OpenImage {
    pub img: RgbImage,
    pub dims: (u32, u32), //(width, height)
    pub buffer: std::vec::Vec<u8>,
}

#[derive(Debug)]
pub struct AdjacentPixels {
    pub left: (u32, u32),
    pub right: (u32, u32),
    pub up: (u32, u32),
    pub down: (u32, u32),
}

impl PartialEq for AdjacentPixels {
    fn eq(&self, other: &AdjacentPixels) -> bool {
        self.left == other.left
            && self.right == other.right
            && self.up == other.up
            && self.down == other.down
    }
}

impl OpenImage {
    pub fn pixel_energy(&self, pos: (u32, u32)) -> i32 {
        let px = self.img.get_pixel(pos.0, pos.1);
        println!("The pixel RGB Vals are {}, {}, {}", px[0], px[1], px[2]);
        let adj_px = self.get_adjacent_pixels(pos);

        let rx = self.img.get_pixel(adj_px.left.0, adj_px.left.1)[0] as i32
            - self.img.get_pixel(adj_px.right.0, adj_px.right.1)[0] as i32;
        let gx = self.img.get_pixel(adj_px.left.0, adj_px.left.1)[1] as i32
            - self.img.get_pixel(adj_px.right.0, adj_px.right.1)[1] as i32;
        let bx = self.img.get_pixel(adj_px.left.0, adj_px.left.1)[2] as i32
            - self.img.get_pixel(adj_px.right.0, adj_px.right.1)[2] as i32;

        let delta_x_squared = (rx * rx) + (gx * gx) + (bx * bx);
        println!("dx^2: {}", delta_x_squared);

        let ry = self.img.get_pixel(adj_px.up.0, adj_px.up.1)[0] as i32
            - self.img.get_pixel(adj_px.down.0, adj_px.down.1)[0] as i32;
        let gy = self.img.get_pixel(adj_px.up.0, adj_px.up.1)[1] as i32
            - self.img.get_pixel(adj_px.down.0, adj_px.down.1)[1] as i32;
        let by = self.img.get_pixel(adj_px.up.0, adj_px.up.1)[2] as i32
            - self.img.get_pixel(adj_px.down.0, adj_px.down.1)[2] as i32;

        let delta_y_squared = (ry * ry) + (gy * gy) + (by * by);
        println!("dy^2: {}", delta_y_squared);

        delta_x_squared + delta_y_squared
    }

    fn get_adjacent_pixels(&self, pos: (u32, u32)) -> AdjacentPixels {
        let (x, y) = pos;
        let x_left = {
            if x == 0 {
                self.dims.0 - 1
            } else {
                x - 1
            }
        };
        let x_right = {
            if x == self.dims.0 - 1 {
                0
            } else {
                x + 1
            }
        };
        let y_up = {
            if y == 0 {
                self.dims.1 - 1
            } else {
                y - 1
            }
        };
        let y_down = {
            if y == self.dims.1 - 1 {
                0
            } else {
                y + 1
            }
        };

        AdjacentPixels {
            left: (x_left, pos.1),
            right: (x_right, pos.1),
            up: (pos.0, y_up),
            down: (pos.0, y_down),
        }
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
    let buffer = img.to_vec();
    let opened_image = OpenImage { img, dims, buffer };

    Ok(())
}
