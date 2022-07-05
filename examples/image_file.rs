extern crate image;
extern crate reduced_color;

use std::fs::*;
use std::path::*;
use std::io::BufReader;

use reduced_color::{ColorNode, MedianCut};

const QUANT_SIZE: u32 = 8;

fn main() {
    let paths = read_dir("./examples/res");

    for path in paths.unwrap() {
        let path = path.unwrap().path();
        let path = path.to_str().unwrap();
        process_image(path);
    }
}

fn process_image(file: &str) {
    println!("Reading image {}", file);

    let img = image::load(BufReader::new(File::open(file).unwrap()), image::ImageFormat::Png).unwrap();

    let mut image = img.to_rgba8();
//    let median_cut = MedianCut::from_pixels_u8_rgba(&image, QUANT_SIZE);
    let mut colors: Vec<ColorNode> = Vec::new();
    colors.push(ColorNode::from(0, 0, 0, 0));
    colors.push(ColorNode::from(255, 255, 255, 0));
    colors.push(ColorNode::from(255, 0, 0, 0));
    let median_cut = MedianCut::new(colors);

    println!("Quantized {:?}", median_cut.colors());

    median_cut.quantize_image_from(&mut image);

    let ref outfile = format!("./target/{}.png",
                              Path::new(file).file_stem().unwrap().to_str().unwrap());

    let _ = image.save_with_format(outfile, image::ImageFormat::Png);
}
