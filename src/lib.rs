use std::error::Error;
use std::ops::Deref;
use std::path::Path;
use std::process;

use image::{ImageBuffer, Pixel, RgbImage};
use imageproc::gradients;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_image_dimensions() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .to_rgb();
        let dims = img.dimensions();
        let buffer: Vec<Vec<[u8; 3]>> = formatted_buffer(&img);
        let opened_image = OpenImage { img, dims, buffer };

        assert_eq!((1024, 694), opened_image.dims);
    }

    #[test]
    fn check_image_buffer_dimensions() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .to_rgb();
        let dims = img.dimensions();
        let buffer: Vec<Vec<[u8; 3]>> = formatted_buffer(&img);
        let opened_image = OpenImage { img, dims, buffer };
        assert_eq!(
            (694, 1024),
            (
                opened_image.buffer.len(),
                opened_image.buffer.first().unwrap().len()
            )
        );
    }

    #[test]
    fn check_length_after_seam_removal() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .to_rgb();
        let dims = img.dimensions();
        let buffer: Vec<Vec<[u8; 3]>> = formatted_buffer(&img);
        let mut opened_image = OpenImage { img, dims, buffer };

        let seam = find_vertical_seam(&opened_image);
        opened_image.remove_vertical_seam(seam);

        assert_eq!(
            (694, 1023),
            (
                opened_image.buffer.len(),
                opened_image.buffer.first().unwrap().len()
            )
        );
    }

    #[test]
    fn multiple_seam_removals() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .to_rgb();
        let dims = img.dimensions();
        let buffer: Vec<Vec<[u8; 3]>> = formatted_buffer(&img);
        let mut opened_image = OpenImage { img, dims, buffer };

        let num_seams_to_remove = 20;

        for _ in 1..=num_seams_to_remove {
            let seam = find_vertical_seam(&opened_image);
            opened_image.remove_vertical_seam(seam);
        }

        image::save_buffer(
            Path::new(&"images/seam_test1_test.jpg"),
            &opened_image.buffer.concat().concat(),
            opened_image.dims.0,
            opened_image.dims.1,
            image::RGB(8),
        )
        .unwrap();

        assert_eq!(
            (694, 1004),
            (
                opened_image.buffer.len(),
                opened_image.buffer.first().unwrap().len()
            )
        );
    }

    #[test]
    fn print_rgb_first_row() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .to_rgb();
        let dims = img.dimensions();
        let buffer: Vec<Vec<[u8; 3]>> = formatted_buffer(&img);
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
        let buffer: Vec<Vec<[u8; 3]>> = formatted_buffer(&img);
        let opened_image = OpenImage { img, dims, buffer };

        assert_eq!(33822, opened_image.pixel_energy((0, 0)));
    }

    #[test]
    fn adjacency_test_corner_pixels() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .to_rgb();
        let dims = img.dimensions();
        let buffer: Vec<Vec<[u8; 3]>> = formatted_buffer(&img);
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

    #[test]
    fn check_lower_edge_pixels() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .to_rgb();
        let dims = img.dimensions();
        let buffer: Vec<Vec<[u8; 3]>> = formatted_buffer(&img);
        let opened_image = OpenImage { img, dims, buffer };

        println!("{:?}", opened_image.get_lower_edges((3, 4)).unwrap());
        println!("{:?}", opened_image.get_lower_edges((0, 0)).unwrap());
    }

    #[test]
    fn find_min_energy() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .to_rgb();
        let dims = img.dimensions();
        let buffer: Vec<Vec<[u8; 3]>> = formatted_buffer(&img);
        let oi = OpenImage { img, dims, buffer };
        let first_row_energy: Vec<i32> = oi
            .img
            .rows()
            .enumerate()
            .map(|(i, _)| oi.pixel_energy((i as u32, 0)))
            .collect();

        // Get the minimum pixel energy in the first row
        let min_energy_pixel = first_row_energy.iter().min().unwrap();
        let min_energy_pixel_pos = first_row_energy
            .iter()
            .position(|&x| x == *min_energy_pixel)
            .unwrap() as u32;

        println!(
            "{:?}, {:?}, {:?}",
            min_energy_pixel, min_energy_pixel_pos, 0
        );

        println!("{:?}", oi.get_lower_edges((min_energy_pixel_pos, 0)));
    }

    #[test]
    fn put_red_px() {
        let img = image::open(Path::new("images/test_image1.png"))
            .unwrap()
            .to_rgb();
        let dims = img.dimensions();
        let buffer: Vec<Vec<[u8; 3]>> = formatted_buffer(&img);
        let mut oi = OpenImage { img, dims, buffer };
        for _ in 1..=80 {
            let seam = find_vertical_seam(&oi);
            oi.remove_vertical_seam(seam);
        }

        println!("{} by {}", oi.dims.0, oi.dims.1);

        image::save_buffer(
            Path::new(&"images/seam_test1.jpg"),
            &oi.buffer.concat().concat(),
            oi.dims.0,
            oi.dims.1,
            image::RGB(8),
        )
        .unwrap();
    }

    #[test]
    fn test_rgb_conversions() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .grayscale();

        let sg = imageproc::gradients::sobel_gradients(&img.as_luma8().unwrap());
        println!("{:?}", sg);
        // img.save(Path::new("images/test_image_gray.jpg")).unwrap();
        // img.as_rgb8()
        //     .unwrap()
        //     .save(Path::new("images/test_image_rgb.jpg"))
        //     .unwrap();
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
    pub buffer: Vec<Vec<[u8; 3]>>,
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
        // let px = self.img.get_pixel(pos.0, pos.1);
        // println!("The pixel RGB Vals are {}, {}, {}", px[0], px[1], px[2]);
        let adj_px = self.get_adjacent_pixels(pos);

        let rx = self.img.get_pixel(adj_px.left.0, adj_px.left.1)[0] as i32
            - self.img.get_pixel(adj_px.right.0, adj_px.right.1)[0] as i32;
        let gx = self.img.get_pixel(adj_px.left.0, adj_px.left.1)[1] as i32
            - self.img.get_pixel(adj_px.right.0, adj_px.right.1)[1] as i32;
        let bx = self.img.get_pixel(adj_px.left.0, adj_px.left.1)[2] as i32
            - self.img.get_pixel(adj_px.right.0, adj_px.right.1)[2] as i32;

        let delta_x_squared = (rx * rx) + (gx * gx) + (bx * bx);
        // println!("dx^2: {}", delta_x_squared);

        let ry = self.img.get_pixel(adj_px.up.0, adj_px.up.1)[0] as i32
            - self.img.get_pixel(adj_px.down.0, adj_px.down.1)[0] as i32;
        let gy = self.img.get_pixel(adj_px.up.0, adj_px.up.1)[1] as i32
            - self.img.get_pixel(adj_px.down.0, adj_px.down.1)[1] as i32;
        let by = self.img.get_pixel(adj_px.up.0, adj_px.up.1)[2] as i32
            - self.img.get_pixel(adj_px.down.0, adj_px.down.1)[2] as i32;

        let delta_y_squared = (ry * ry) + (gy * gy) + (by * by);
        // println!("dy^2: {}", delta_y_squared);

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

    // for a given index (col, row), this yields a vector of the indexes of the pixels directly
    // below it
    // Example:
    // the lower edges for an index (3, 4) would give (2, 5), (3, 5), (4, 5)
    fn get_lower_edges(&self, pos: (u32, u32)) -> Result<(u32, Vec<(u32, u32)>), &'static str> {
        let down_row = pos.1 + 1;
        let mut down_indices: Vec<(u32, u32)> = Vec::new();

        if down_row == self.dims.1 - 1 {
            return Err("Last row reached");
        } else {
            let possibilities = vec![(pos.0, down_row), (pos.0 + 1, down_row)];
            if pos.0 > 0 {
                down_indices.push((pos.0 - 1, down_row));
            }
            for (col, _) in possibilities {
                if col <= self.dims.1 - 1 {
                    down_indices.push((col, down_row));
                }
            }

            Ok((down_row, down_indices))
        }
    }

    // TODO
    // This method needs to be modified to also remove the seam from the energy buffer data structure
    fn remove_vertical_seam(&mut self, v_seam: std::vec::Vec<(u32, u32)>) {
        for (col, row) in v_seam {
            if let Some(elem) = self.buffer.get_mut(row as usize) {
                elem.remove(col as usize);
                elem.shrink_to_fit();
            }
        }
        self.dims.0 = self.dims.0 - 1;
    }
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        args.next();

        let img_path = match args.next() {
            Some(arg) => arg,
            None => return Err("No filename specified"),
        };

        let reduce_by = match args.next() {
            Some(arg) => arg.parse().unwrap_or_else(|err| {
                eprintln!(
                    "Couldn't parse the provided argument as an integer: {}",
                    err
                );
                process::exit(1);
            }),
            None => return Err("Missing resize percentage"),
        };

        Ok(Config {
            img_path,
            reduce_by,
        })
    }
}

fn find_vertical_seam(oi: &OpenImage) -> std::vec::Vec<(u32, u32)> {
    let mut computed_seam: Vec<(u32, u32)> = Vec::new();

    // Compute the pixel energies for the first row
    let first_row_energy: Vec<i32> = oi
        .img
        .rows()
        .enumerate()
        .map(|(i, _)| oi.pixel_energy((i as u32, 0)))
        .collect();

    // Get the minimum pixel energy in the first row
    let min_energy_pixel = first_row_energy.iter().min().unwrap();
    let min_energy_pixel_pos = first_row_energy
        .iter()
        .position(|&x| x == *min_energy_pixel)
        .unwrap() as u32;

    computed_seam.push((min_energy_pixel_pos, 0));

    loop {
        match oi.get_lower_edges(*computed_seam.last().unwrap()) {
            Ok((_, v)) => {
                // println!("{:?}", (&r, &v));
                let row_energy: Vec<i32> =
                    v.iter().map(|(i, j)| oi.pixel_energy((*i, *j))).collect();

                let min_energy_pixel = row_energy.iter().min().unwrap();
                let min_energy_pixel_pos = row_energy
                    .iter()
                    .position(|&x| x == *min_energy_pixel)
                    .unwrap();
                computed_seam.push(*v.get(min_energy_pixel_pos).unwrap());
            }
            Err(_) => break,
        }
    }
    computed_seam
}

fn formatted_buffer<P, Container>(img: &ImageBuffer<P, Container>) -> Vec<Vec<[u8; 3]>>
where
    P: Pixel<Subpixel = u8> + 'static,
    Container: Deref<Target = [u8]>,
{
    let mut buffer: Vec<Vec<[u8; 3]>> = Vec::new();
    for (_, im) in img.enumerate_rows() {
        let mut row_vec: Vec<[u8; 3]> = Vec::new();
        for (_i, (_, _, px)) in im.enumerate() {
            let px_chans = px.channels();
            row_vec.push([px_chans[0], px_chans[1], px_chans[2]]);
        }
        buffer.push(row_vec);
    }

    buffer
}

fn cumulative_energy(buff: &Vec<Vec<[u8; 3]>>) -> Vec<Vec<[u8; 3]>> {
    for j in 1..=buff.len() {}
    buff.clone() // placeholder to remove return type error
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let img_base = image::open(Path::new(&config.img_path))?;

    let sg = gradients::sobel_gradients(&img_base.as_luma8().unwrap());
    println!("{:?}", sg);

    let img = img_base.to_rgb();
    let dims = img.dimensions();

    // The buffer is now a formatted object that contains the gradient magnitude
    // values for each pixel
    // We can then compute the cumulative energy for the buffer to use in our
    // seam calculations
    let buffer: Vec<Vec<[u8; 3]>> = formatted_buffer(&img);
    let mut opened_image = OpenImage { img, dims, buffer };
    let seam_iterations: u32 =
        (opened_image.dims.0 as f32 * (config.reduce_by as f32 / 100.)) as u32;
    println!("{}", seam_iterations);

    for _ in 1..=seam_iterations {
        let seam = find_vertical_seam(&opened_image);
        opened_image.remove_vertical_seam(seam);
    }

    image::save_buffer(
        Path::new(&"images/seam_test1.jpg"),
        &opened_image.buffer.concat().concat(),
        opened_image.dims.0,
        opened_image.dims.1,
        image::RGB(8),
    )?;
    println!("Seam... Carved");

    Ok(())
}
