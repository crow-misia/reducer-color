use std::cmp::{max, min};
use std::collections::HashMap;
use image::{Pixel, RgbaImage};
use crate::ColorNode;

enum ColorDimension {
    Red,
    Green,
    Blue,
}

#[derive(Debug)]
struct Box {
    lower: usize,
    upper: usize,
    count: usize,
    level: usize,
}

impl Box {
    fn new(lower: usize, upper: usize, level: usize) -> Box {
        Box {
            lower,
            upper,
            level,
            count: 0,
        }
    }

    fn color_count(&self) -> usize {
        self.upper - self.lower
    }

    fn split_box(&mut self, pixels: &mut Vec<ColorNode>) -> Option<Box> {
        if self.color_count() < 2 {
            None
        } else {
            let dim = self.get_longest_color_dimension(pixels);

            let med = self.find_median(dim, pixels);

            // now split this box at the median return the resulting new box.
            let next_level = self.level + 1;
            let new_box = Box::new(med + 1, self.upper, next_level);
            self.upper = med;
            self.level = next_level;
            Some(new_box)
        }
    }

    fn get_longest_color_dimension(&mut self, pixels: &Vec<ColorNode>) -> ColorDimension {
        let mut min_r = 255;
        let mut max_r = 0;
        let mut min_g = 255;
        let mut max_g = 0;
        let mut min_b = 255;
        let mut max_b = 0;
        let mut count = 0;

        for i in self.lower..self.upper {
            let pixel = &pixels[i];
            count += pixel.uses;
            let r = pixel.r as i32;
            let g = pixel.g as i32;
            let b = pixel.b as i32;
            min_r = min(r, min_r);
            max_r = max(r, max_r);
            min_g = min(g, min_g);
            max_g = max(g, max_g);
            min_b = min(b, min_b);
            max_b = max(b, max_b);
        }

        self.count = count;

        let count_r = max_r - min_r;
        let count_g = max_g - min_g;
        let count_b = max_b - max_b;

        if count_b >= count_r && count_b >= count_g {
            ColorDimension::Blue
        } else if count_g >= count_r && count_g >= count_b {
            return ColorDimension::Green;
        } else {
            ColorDimension::Red
        }
    }

    fn find_median(&self, dim: ColorDimension, pixels: &mut Vec<ColorNode>) -> usize {
        match dim {
            ColorDimension::Red => pixels[self.lower..(self.upper + 1)].sort_by(|a, b| a.r.cmp(&b.r)),
            ColorDimension::Green => pixels[self.lower..(self.upper + 1)].sort_by(|a, b| a.g.cmp(&b.g)),
            ColorDimension::Blue => pixels[self.lower..(self.upper + 1)].sort_by(|a, b| a.b.cmp(&b.b)),
        }

        let half = self.count / 2;
        let mut uses = 0;
        for median in self.lower..self.upper {
            uses += pixels[median].uses;
            if uses >= half {
                return median;
            }
        }
        self.lower
    }

    fn get_average_color(&self, pixels: &Vec<ColorNode>) -> ColorNode {
        let mut a_sum = 0;
        let mut r_sum = 0;
        let mut g_sum = 0;
        let mut b_sum = 0;
        let mut n = 0usize;
        for pixel in pixels[self.lower..(self.upper + 1)].iter() {
            let cnt = pixel.uses;
            a_sum += cnt * pixel.a as usize;
            r_sum += cnt * pixel.r as usize;
            g_sum += cnt * pixel.g as usize;
            b_sum += cnt * pixel.b as usize;
            n += cnt;
        }
        let nf = n as f64;
        let avg_a = (0.5 + a_sum as f64 / nf) as u8;
        let avg_r = (0.5 + r_sum as f64 / nf) as u8;
        let avg_g = (0.5 + g_sum as f64 / nf) as u8;
        let avg_b = (0.5 + b_sum as f64 / nf) as u8;
        ColorNode::from_color(avg_a, avg_r, avg_g, avg_b, n)
    }
}


pub struct MedianCut { }

impl MedianCut {
    pub fn from_pixels_u8_rgba(image: &RgbaImage, k_max: u32) -> Vec<ColorNode> {
        let mut quant_colors = MedianCut::find_representative_colors(image, k_max);
        quant_colors.sort_by(|a, b| b.uses.cmp(&a.uses));

        quant_colors
    }

    pub fn quantize_image_from(colors: &Vec<ColorNode>, image: &mut RgbaImage) {
        image.pixels_mut().for_each(|pixel| {
            let rgba = pixel.channels();
            let c = MedianCut::find_closest_color(colors, ColorNode::convert_argb_to_u32(rgba[3], rgba[2], rgba[1], rgba[0]));
            let color = ColorNode::convert_u32_to_argb(c);
            pixel[0] = color[0];
            pixel[1] = color[1];
            pixel[2] = color[2];
            pixel[3] = color[3];
        })
    }

    pub fn create_histogram(image: &RgbaImage) -> Vec<ColorNode> {
        let mut count: HashMap<u32, usize> = HashMap::new();
        image.pixels()
            .for_each(|pixel| {
                let channels = pixel.channels();
                let color = ColorNode::convert_argb_to_u32(channels[3], channels[2], channels[1], channels[0]);
                *count.entry(color).or_insert(0) += 1;
            });
        count.into_iter().map(|(p, c)| ColorNode::from_abgr(p, c)).collect()
    }

    fn find_representative_colors(pixels: &RgbaImage, k_max: u32) -> Vec<ColorNode> {
        // create color histogram
        let mut pixels: Vec<_> = MedianCut::create_histogram(pixels);
        let color_num = pixels.len();

        let r_cols = if color_num <= k_max as usize {
            // image has fewer colors than k_max
            pixels
        } else {
            let initial_box = Box::new(0, color_num - 1, 0);
            let mut color_set = Vec::new();
            color_set.push(initial_box);
            let mut k = 1;
            let mut done = false;
            while k < k_max && !done {
                let new_box = if let Some(next_box) = MedianCut::find_box_to_split(&mut color_set) {
                    next_box.split_box(&mut pixels)
                } else {
                    done = true;
                    None
                };

                if let Some(new_box) = new_box {
                    color_set.push(new_box);
                    k += 1;
                }
            }

            MedianCut::average_colors(&color_set, &pixels)
        };
        r_cols
    }

    fn find_closest_color(colors: &Vec<ColorNode>, rgb: u32) -> u32 {
        let r = (rgb & 0xff) as u8;
        let g = ((rgb & 0xff00) >> 8) as u8;
        let b = ((rgb & 0xff0000) >> 16) as u8;
        let mut min_idx = 0;
        let mut min_distance = ::std::i32::MAX;
        for (i, color) in colors.iter().enumerate() {
            let d = color.distance2(r, g, b);
            if d < min_distance {
                min_distance = d;
                min_idx = i;
            }
        }
        colors[min_idx].to_abgr()
    }

    fn average_colors(boxes: &Vec<Box>, pixels: &Vec<ColorNode>) -> Vec<ColorNode> {
        let n = boxes.len();
        let mut avg_colors = Vec::with_capacity(n);
        for b in boxes {
            avg_colors.push(b.get_average_color(&pixels));
        }
        return avg_colors;
    }

    fn find_box_to_split(boxes: &mut Vec<Box>) -> Option<&mut Box> {
        let mut box_to_split = None;
        let mut min_level = ::std::usize::MAX;
        for b in boxes {
            if b.color_count() >= 2 {
                if b.level < min_level {
                    min_level = b.level;
                    box_to_split = Some(b);
                }
            }
        }
        box_to_split
    }
}
