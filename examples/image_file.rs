extern crate image;
extern crate reduced_color;

use std::fs::*;
use std::path::*;
use std::io::BufReader;

use reduced_color::MedianCut;
use image::Pixel;

const QUANT_SIZE: u32 = 16;

fn main() {
    let paths = read_dir("./examples/res");

    for path in paths.unwrap() {
        let path = path.unwrap().path();
        let path = path.to_str().unwrap();
        process_image(path);
    }

    println!("\nPlease visit 'target' folder for the results");
}

fn convert_to_pixel(pixel: &image::Rgba<u8>) -> u32 {
    let rgba = pixel.channels();
    reduced_color::ColorNode::convert_argb_to_u32(rgba[3], rgba[2], rgba[1], rgba[0])
}

fn process_image(file: &str) {
    println!("Reading image {}", file);

    let img = image::load(BufReader::new(File::open(file).unwrap()), image::ImageFormat::Png).unwrap();


    // Here we extract the quantized colors from the image.
    // We need no more than 16 colors (QUANT_SIZE).
    let mut image = img.to_rgba8();
    let colors = MedianCut::from_pixels_u8_rgba(&image, QUANT_SIZE);

    // A `Vec` of colors, descendantely sorted by usage frequency
    // let qc = mcq.get_quantized_colors();
/*    colors = Vec::with_capacity(3);
    colors.push(reduced_color::ColorNode::from_color(0xff, 0xff, 0, 0, 1));
    colors.push(reduced_color::ColorNode::from_color(0xff, 0xff, 0xff,0xff,1));
    colors.push(reduced_color::ColorNode::from_color(0xff, 0, 0, 0, 1));
*/    println!("Quantized {:?}", colors);

    // =============================================================================================
    // Here we will demonstrate the extracted colors by generating the image
    // that consists of both original image and a resulted palette.
    // =============================================================================================
    // let mut img = image::load(BufReader::new(File::open(file).unwrap()), image::ImageFormat::Png)
    //     .unwrap();

    MedianCut::quantize_image_from(&colors, &mut image);

/*    let color_width = ix / QUANT_SIZE;

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
*/
    let ref outfile = format!("./target/{}.png",
                              Path::new(file).file_stem().unwrap().to_str().unwrap());

    let _ = img.save_with_format(outfile, image::ImageFormat::Png);
}
