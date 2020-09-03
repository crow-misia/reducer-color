extern crate image;
extern crate reduced_color;

use std::fs::*;
use std::path::*;
use std::io::BufReader;

use reduced_color::MedianCut;

const COLOR_HEIGHT: u32 = 64;
const QUANT_SIZE: u32 = 256;

fn main() {
    let paths = read_dir("./examples/res");

    for path in paths.unwrap() {
        let path = path.unwrap().path();
        let path = path.to_str().unwrap();
        process_image(path);
    }

    println!("\nPlease visit 'target' folder for the results");
}

fn process_image(file: &str) {
    println!("Reading image {}", file);

    let img = image::load(BufReader::new(File::open(file).unwrap()), image::ImageFormat::Png).unwrap().to_rgba();
    let data = img.into_vec();

    // Here we extract the quantized colors from the image.
    // We need no more than 16 colors (QUANT_SIZE).
    let mcq = MedianCut::from_pixels_u8_rgba(data.as_slice(), QUANT_SIZE);

    // A `Vec` of colors, descendantely sorted by usage frequency
    let qc = mcq.get_quantized_colors();
    println!("Quantized {:?}", qc);

    // =============================================================================================
    // Here we will demonstrate the extracted colors by generating the image
    // that consists of both original image and a resulted palette.
    // =============================================================================================
    let img = image::load(BufReader::new(File::open(file).unwrap()), image::ImageFormat::Png).unwrap().to_rgba();

    let (ix, iy) = img.dimensions();

    let mut imgbuf = image::ImageBuffer::new(ix, iy + COLOR_HEIGHT);
    let quantize_img = mcq.quantize_image_u8_rgba(img.into_vec().as_slice());
    let mcq = MedianCut::from_pixels_u8_rgba(quantize_img, QUANT_SIZE);

  //  imgbuf.copy_from_slice(quantize_img);

    let color_width = ix / QUANT_SIZE;

    for x0 in 0..QUANT_SIZE {
        let x1 = x0 * color_width;
        let q = qc[x0 as usize];
        let c = image::Rgba([q.r, q.g, q.b, 0xff]);

        for y in (iy + 1)..(iy + COLOR_HEIGHT) {
            for x2 in 0..color_width {
                imgbuf.put_pixel(x1 + x2, y, c);
            }
        }
    }

    let ref outfile = format!("./target/{}.png",
                              Path::new(file).file_stem().unwrap().to_str().unwrap());

    let _ = image::DynamicImage::ImageRgba8(imgbuf).save_with_format(outfile, image::ImageFormat::Png);
}