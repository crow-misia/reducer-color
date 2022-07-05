use std::cmp::{max, min};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use image::{Pixel, Rgba, RgbaImage};

enum ColorDimension {
    Red,
    Green,
    Blue,
}

#[derive(Debug, Clone, Copy)]
pub struct ColorNode {
    pub rgb: u32,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub count: usize,
}

impl Hash for ColorNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.rgb)
    }
}

impl PartialEq for ColorNode {
    fn eq(&self, other: &Self) -> bool {
        self.rgb == other.rgb
    }
}

impl ColorNode {
    #[inline]
    pub fn from_rgb(rgb: u32, count: usize) -> ColorNode {
        ColorNode {
            rgb,
            blue: (rgb & 0xff0000 >> 16) as u8,
            green: (rgb & 0xff00 >> 8) as u8,
            red: (rgb & 0xff) as u8,
            count,
        }
    }

    #[inline]
    pub fn from(red: u8, green: u8, blue: u8, count: usize) -> ColorNode {
        ColorNode {
            rgb: ColorNode::convert_rgb_to_u32(red, green, blue),
            blue,
            green,
            red,
            count,
        }
    }

    #[inline]
    fn convert_rgb_to_u32(red: u8, green: u8, blue: u8) -> u32 {
        (red as u32) << 16 | (green as u32) << 8 | blue as u32
    }

    fn distance2(&self, other: &ColorNode) -> i32 {
        let dr = self.red as i32 - other.red as i32;
        let dg = self.green as i32 - other.green as i32;
        let db = self.blue as i32 - other.blue as i32;
        return dr * dr + dg * dg + db * db;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct ColorBox {
    lower: usize,
    upper: usize,
    level: isize,
    count: usize,
    min_r: i32,
    max_r: i32,
    min_g: i32,
    max_g: i32,
    min_b: i32,
    max_b: i32,
}

impl ColorBox {
    fn new(lower: usize, upper: usize, level: isize, colors: &Vec<ColorNode>) -> ColorBox {
        let mut b = ColorBox {
            lower,
            upper,
            level,
            ..Default::default()
        };
        b.trim(colors);
        b
    }

    fn color_count(&self) -> usize {
        self.upper - self.lower
    }

    fn trim(&mut self, colors: &Vec<ColorNode>) {
        let mut min_r = 255;
        let mut max_r = 0;
        let mut min_g = 255;
        let mut max_g = 0;
        let mut min_b = 255;
        let mut max_b = 0;
        let mut count = 0;

        for i in self.lower..self.upper {
            let color = &colors[i];
            count += color.count;
            let r = color.red as i32;
            let g = color.green as i32;
            let b = color.blue as i32;
            min_r = min(r, min_r);
            max_r = max(r, max_r);
            min_g = min(g, min_g);
            max_g = max(g, max_g);
            min_b = min(b, min_b);
            max_b = max(b, max_b);
        }
        self.min_r = min_r;
        self.max_r = max_r;
        self.min_g = min_g;
        self.max_g = max_g;
        self.min_b = min_b;
        self.max_b = max_b;
        self.count = count;
    }

    fn split_box(&mut self, colors: &mut Vec<ColorNode>) -> Option<ColorBox> {
        if self.color_count() < 2 {
            None
        } else {
            let dim = self.get_longest_color_dimension();

            let med = self.find_median(dim, colors);

            let next_level = self.level + 1;
            let new_box = ColorBox::new(med + 1, self.upper, next_level, colors);
            self.upper = med;
            self.level = next_level;
            self.trim(colors);
            Some(new_box)
        }
    }

    fn get_longest_color_dimension(&self) -> ColorDimension {
        let count_r = self.max_r - self.min_r;
        let count_g = self.max_g - self.min_g;
        let count_b = self.max_b - self.max_b;

        if count_b >= count_r && count_b >= count_g {
            ColorDimension::Blue
        } else if count_g >= count_r && count_g >= count_b {
            ColorDimension::Green
        } else {
            ColorDimension::Red
        }
    }

    fn find_median(&self, dim: ColorDimension, colors: &mut Vec<ColorNode>) -> usize {
        match dim {
            ColorDimension::Red => colors[self.lower..(self.upper + 1)].sort_by(|a, b| a.red.cmp(&b.red)),
            ColorDimension::Green => colors[self.lower..(self.upper + 1)].sort_by(|a, b| a.green.cmp(&b.green)),
            ColorDimension::Blue => colors[self.lower..(self.upper + 1)].sort_by(|a, b| a.blue.cmp(&b.blue)),
        }

        let half = self.count / 2;
        let mut uses = 0;
        for median in self.lower..self.upper {
            uses += colors[median].count;
            if uses >= half {
                return median;
            }
        }
        self.lower
    }

    fn get_average_color(&self, colors: &Vec<ColorNode>) -> ColorNode {
        let mut sum_r = 0;
        let mut sum_g = 0;
        let mut sum_b = 0;
        let mut n = 0usize;
        for color in colors[self.lower..(self.upper + 1)].iter() {
            let count = color.count;
            sum_r += count * color.red as usize;
            sum_g += count * color.green as usize;
            sum_b += count * color.blue as usize;
            n += count;
        }
        let avg_r = (0.5 + sum_r as f64 / n as f64) as u8;
        let avg_g = (0.5 + sum_g as f64 / n as f64) as u8;
        let avg_b = (0.5 + sum_b as f64 / n as f64) as u8;
        ColorNode::from(avg_r, avg_g, avg_b, n)
    }
}

pub struct MedianCut {
    colors: Vec<ColorNode>,
}

impl MedianCut {
    pub fn new(colors: Vec<ColorNode>) -> MedianCut {
        MedianCut {
            colors,
        }
    }

    pub fn from_pixels_u8_rgba(image: &RgbaImage, k_max: u32) -> MedianCut {
        let mut quant_colors = MedianCut::find_representative_colors(image, k_max);
        quant_colors.sort_by(|a, b| b.count.cmp(&a.count));

        MedianCut {
            colors: quant_colors,
        }
    }

    pub fn quantize_image_from(&self, image: &mut RgbaImage) {
        for pixel in image.pixels_mut() {
            let rgba = pixel.channels();
            let color = self.find_closest_color(&ColorNode::from(rgba[0], rgba[1], rgba[2], 0));
            *pixel = Rgba::from([color.red, color.green, color.blue, 255]);
        }
    }

    pub fn create_histogram(image: &RgbaImage) -> Vec<ColorNode> {
        let mut count: HashMap<u32, usize> = HashMap::new();
        image.pixels()
            .for_each(|pixel| {
                let channels = pixel.channels();
                let color = ColorNode::convert_rgb_to_u32(channels[2], channels[1], channels[0]);
                *count.entry(color).or_insert(0) += 1;
            });
        count.into_iter().map(|(p, c)| ColorNode::from_rgb(p, c)).collect()
    }

    pub fn colors(&self) -> Vec<ColorNode> {
        self.colors.to_vec()
    }

    fn find_representative_colors(image: &RgbaImage, k_max: u32) -> Vec<ColorNode> {
        let mut colors = MedianCut::create_histogram(image);
        let color_num = colors.len();

        let r_cols = if color_num <= k_max as usize {
            // image has fewer colors than k_max
            colors
        } else {
            let initial_box = ColorBox::new(0, color_num - 1, 0, &colors);
            let mut color_set = Vec::new();
            color_set.push(initial_box);
            let mut k = 1;
            let mut done = false;
            while k < k_max && !done {
                let new_box = if let Some(next_box) = MedianCut::find_box_to_split(&mut color_set) {
                    next_box.split_box(&mut colors)
                } else {
                    done = true;
                    None
                };

                if let Some(new_box) = new_box {
                    color_set.push(new_box);
                    k += 1;
                }
            }

            MedianCut::average_colors(&color_set, &colors)
        };
        r_cols
    }

    fn find_closest_color(&self, color: &ColorNode) -> ColorNode {
        let mut min_idx = 0;
        let mut min_distance = i32::MAX;
        for (i, other) in self.colors.iter().enumerate() {
            let d = other.distance2(color);
            if d < min_distance {
                min_distance = d;
                min_idx = i;
            }
        }
        self.colors[min_idx]
    }

    fn average_colors(boxes: &Vec<ColorBox>, pixels: &Vec<ColorNode>) -> Vec<ColorNode> {
        let n = boxes.len();
        let mut avg_colors = Vec::with_capacity(n);
        for b in boxes {
            avg_colors.push(b.get_average_color(&pixels));
        }
        return avg_colors;
    }

    fn find_box_to_split(boxes: &mut Vec<ColorBox>) -> Option<&mut ColorBox> {
        let mut box_to_split = None;
        let mut min_level = isize::MAX;
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
