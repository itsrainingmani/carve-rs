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

    #[test]
    fn check_lower_edge_pixels() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .to_rgb();
        let dims = img.dimensions();
        let buffer = img.to_vec();
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
        let buffer = img.to_vec();
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
            Ok((r, v)) => {
                println!("{:?}", (&r, &v));
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

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let img = image::open(Path::new(&config.img_path))?.to_rgb();
    let dims = img.dimensions();
    let buffer = img.to_vec();
    let opened_image = OpenImage { img, dims, buffer };

    let mut test_image = opened_image.img.clone();
    let vert_seam = find_vertical_seam(&opened_image);

    for si in vert_seam {
        let red_px = image::Rgb([255u8, 0u8, 0u8]);
        test_image.put_pixel(si.0, si.1, red_px);
    }

    test_image.save(Path::new(&"images/seam_test.jpg"))?;
    println!("Seam... Carved");

    Ok(())
}
