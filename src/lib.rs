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
        let opened_image = OpenImage::new(&String::from("images/test_image.jpg")).unwrap();
        assert_eq!((1024, 694), opened_image.dims);
    }

    #[test]
    fn check_image_buffer_dimensions() {
        let opened_image = OpenImage::new(&String::from("images/test_image.jpg")).unwrap();
        assert_eq!(
            (694, 1024),
            (
                opened_image.buffer.len(),
                opened_image.buffer.first().unwrap().len()
            )
        );
    }

    #[test]
    fn check_image_energy_dimensions() {
        let opened_image = OpenImage::new(&String::from("images/test_image.jpg")).unwrap();
        assert_eq!(
            (694, 1024),
            (
                opened_image.energy.len(),
                opened_image.energy.first().unwrap().len()
            )
        );
    }

    #[test]
    fn check_length_after_seam_removal() {
        let mut opened_image = OpenImage::new(&String::from("images/test_image.jpg")).unwrap();

        let seam = opened_image.find_vertical_seam();
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
        let mut opened_image = OpenImage::new(&String::from("images/test_image.jpg")).unwrap();
        let num_seams_to_remove = 20;

        for _ in 1..=num_seams_to_remove {
            let seam = opened_image.find_vertical_seam();
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
        let opened_image = OpenImage::new(&String::from("images/test_image.jpg")).unwrap();

        for i in 0..opened_image.dims.0 {
            println!("{:?}", opened_image.img.get_pixel(i, 0));
        }
    }

    #[test]
    fn check_upper_edge_pixels() {
        let opened_image = OpenImage::new(&String::from("images/test_image.jpg")).unwrap();

        println!("{:?}", get_upper_edges(&opened_image.img, (3, 4)).unwrap());
        println!("{:?}", get_upper_edges(&opened_image.img, (0, 1)).unwrap());
    }

    #[test]
    fn find_min_energy() {
        let oi = OpenImage::new(&String::from("images/test_image.jpg")).unwrap();
        // Get the minimum pixel energy in the first row
        let min_energy_pixel = oi.energy.get(1).unwrap().iter().min().unwrap();
        let min_energy_pixel_pos = oi
            .energy
            .get(1)
            .unwrap()
            .iter()
            .position(|&x| x == *min_energy_pixel)
            .unwrap() as u32;

        println!(
            "min_energy_px: {:?}, pos: {:?}",
            min_energy_pixel, min_energy_pixel_pos,
        );
    }

    #[test]
    fn show_cumulative_energy() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .grayscale();

        let sg = gradients::sobel_gradients(&img.as_luma8().unwrap());
        let sobel_buffer = format_grayscale(&sg);
        let oi = OpenImage::new(&String::from("images/test_image.jpg")).unwrap();
        for i in 4..7 {
            println!(
                "Row 0, Col {:?} -> {:?}",
                i,
                oi.energy.get(20).unwrap().get(i).unwrap()
            );
        }
        println!(
            "Row 1, Col 5 -> CumEnergy: {:?}, Gradient: {:?}",
            oi.energy.get(21).unwrap().get(5).unwrap(),
            sobel_buffer.get(21).unwrap().get(5).unwrap()
        );
    }

    #[test]
    fn validate_cumulative_energy() {
        let img = image::open(Path::new("images/test_image.jpg"))
            .unwrap()
            .grayscale();

        let sg = gradients::sobel_gradients(&img.as_luma8().unwrap());
        let sobel_buffer = format_grayscale(&sg);
        let oi = OpenImage::new(&String::from("images/test_image.jpg")).unwrap();
        let upper = oi
            .energy
            .get(20)
            .unwrap()
            .get(4..7)
            .unwrap()
            .iter()
            .min()
            .unwrap();
        assert_eq!(
            *oi.energy.get(21).unwrap().get(5).unwrap(),
            sobel_buffer.get(21).unwrap().get(5).unwrap() + *upper
        );
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

        let energy = cumulative_energy(img_base.grayscale().as_luma8().unwrap());

        Ok(OpenImage {
            img,
            dims,
            buffer,
            energy,
        })
    }

    fn find_vertical_seam(&self) -> Vec<(u32, u32)> {
        let mut computed_seam: Vec<(u32, u32)> = Vec::new();
        // Get the minimum pixel energy in the first row

        let min_energy_pixel = self.energy.last().unwrap().iter().min().unwrap();
        let min_energy_pixel_pos = self
            .energy
            .last()
            .unwrap()
            .iter()
            .position(|&x| x == *min_energy_pixel)
            .unwrap() as u32;
        computed_seam.push((min_energy_pixel_pos, self.dims.1 - 1));
        loop {
            match get_upper_edges(&self.img, *computed_seam.first().unwrap()) {
                Ok((r, v)) => {
                    let energy_row = self.energy.get(r as usize).unwrap();
                    let min_energy_pixel = energy_row.iter().min().unwrap();
                    let min_energy_pixel_pos = energy_row
                        .iter()
                        .position(|&x| x == *min_energy_pixel)
                        .unwrap();
                    computed_seam.insert(0, *v.get(min_energy_pixel_pos).unwrap());
                }
                Err(_) => break,
            }
        }
        computed_seam
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

// for a given index (col, row), this yields a vector of the indexes of the pixels directly
// above it
// Example:
// the upper edges for an index (3, 4) would give (2, 3), (3, 3), (4, 3)
fn get_upper_edges<P, Container>(
    img: &ImageBuffer<P, Container>,
    pos: (u32, u32),
) -> Result<(u32, Vec<(u32, u32)>), &'static str>
where
    P: Pixel<Subpixel = u8> + 'static,
    Container: Deref<Target = [u8]>,
{
    if pos.1 == 0 {
        return Err("First Row");
    } else {
        let up_row = pos.1 - 1;
        let mut up_indices: Vec<(u32, u32)> = Vec::new();
        let dims = img.dimensions();
        let possibilities = vec![(pos.0, up_row), (pos.0 + 1, up_row)];
        if pos.0 > 0 {
            up_indices.push((pos.0 - 1, up_row));
        }
        for (col, _) in possibilities {
            if col <= dims.1 - 1 {
                up_indices.push((col, up_row));
            }
        }

        Ok((up_row, up_indices))
    }
}

fn format_grayscale<P, Container>(sb: &ImageBuffer<P, Container>) -> Vec<Vec<u16>>
where
    P: Pixel<Subpixel = u16> + 'static,
    Container: Deref<Target = [u16]>,
{
    let mut buffer: Vec<Vec<u16>> = Vec::new();
    for (_, im) in sb.enumerate_rows() {
        let mut row_vec: Vec<u16> = Vec::new();
        for (_i, (_, _, px)) in im.enumerate() {
            let px_chans = px.channels();
            row_vec.push(px_chans[0]);
        }
        buffer.push(row_vec);
    }

    buffer
}

fn convert_pos_to_pix<T: Copy>(pos: &[(u32, u32)], buff: &mut Vec<Vec<T>>) -> Vec<T> {
    let mut pxs: Vec<T> = Vec::new();
    for (c, r) in pos {
        pxs.push(*buff.get(*r as usize).unwrap().get(*c as usize).unwrap());
    }

    pxs
}

fn cumulative_energy(gradient: &GrayImage) -> Vec<Vec<u16>> {
    let sobel = gradients::sobel_gradients(gradient);
    let mut buffer = format_grayscale(&sobel);

    let num_rows = buffer.len() - 1;
    let num_cols = buffer.first().unwrap().len() - 1;

    // Start at 1 since the first row's energy does not change
    for r in 1..=num_rows {
        for c in 0..=num_cols {
            let (_, ue) = get_upper_edges(gradient, (c as u32, r as u32)).unwrap();
            let pxs = convert_pos_to_pix(&ue[..], &mut buffer);
            let min_energy = pxs.iter().min().unwrap();
            if let Some(cur_px) = buffer.get_mut(r).unwrap().get_mut(c) {
                *cur_px += min_energy;
            }
        }
    }

    buffer
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut opened_image = OpenImage::new(&config.img_path)?;
    let seam_iterations: u32 =
        (opened_image.dims.0 as f32 * (config.reduce_by as f32 / 100.)) as u32;
    println!("{}", seam_iterations);

    for _ in 1..=seam_iterations {
        let seam = opened_image.find_vertical_seam();
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
