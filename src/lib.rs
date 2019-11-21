use std::error::Error;
use std::ops::Deref;
use std::path::Path;
use std::process;

use image::{GrayImage, ImageBuffer, Pixel, RgbImage};
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
    fn check_upper_edge_pixels() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .to_rgb();
        let dims = img.dimensions();
        let buffer: Vec<Vec<[u8; 3]>> = formatted_buffer(&img);
        let opened_image = OpenImage { img, dims, buffer };

        println!("{:?}", opened_image.get_upper_edges((3, 4)).unwrap());
        println!("{:?}", opened_image.get_upper_edges((1, 0)).unwrap());
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
    pub energy: Vec<Vec<u16>>,
}

impl OpenImage {
    pub fn new(img_path: &String) -> Result<OpenImage, &'static str> {
        let img_base = image::open(Path::new(img_path)).unwrap();

        let img = img_base.to_rgb();
        let dims = img.dimensions();
        let buffer: Vec<Vec<[u8; 3]>> = formatted_buffer(&img);

        let energy = cumulative_energy(&img_base.as_luma8().unwrap());

        Ok(OpenImage {
            img,
            dims,
            buffer,
            energy,
        })
    }

    // for a given index (col, row), this yields a vector of the indexes of the pixels directly
    // above it
    // Example:
    // the upper edges for an index (3, 4) would give (2, 3), (3, 3), (4, 3)
    fn get_upper_edges(&self, pos: (u32, u32)) -> Result<(u32, Vec<(u32, u32)>), &'static str> {
        let up_row = pos.1 - 1;
        let mut up_indices: Vec<(u32, u32)> = Vec::new();

        if up_row == 0 {
            return Err("First row reached");
        } else {
            let possibilities = vec![(pos.0, up_row), (pos.0 + 1, up_row)];
            if pos.0 > 0 {
                up_indices.push((pos.0 - 1, up_row));
            }
            for (col, _) in possibilities {
                if col <= self.dims.1 - 1 {
                    up_indices.push((col, up_row));
                }
            }

            Ok((up_row, up_indices))
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

fn cumulative_energy(gradient: &GrayImage) -> Vec<Vec<u16>> {
    let sobel = gradients::sobel_gradients(gradient);
    println!("{:?}", sobel);
    let mut buffer: Vec<Vec<u16>> = Vec::new();
    for (_, im) in sobel.enumerate_rows() {
        let mut row_vec: Vec<u16> = Vec::new();
        for (_i, (_, _, px)) in im.enumerate() {
            let px_chans = px.channels();
            row_vec.push(px_chans[0]);
        }
        buffer.push(row_vec);
    }

    // for r in 1..=buffer.len() {
    //     let upper_pixels =
    // }
    buffer
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut opened_image = OpenImage::new(&config.img_path)?;
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
