use std::cmp::{max, min};
use std::collections::{HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pixel {
    pub rgb: u32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub uses: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Box {
    lower: usize,
    upper: usize,
    count: usize,
    level: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ColorDimension {
    Red,
    Green,
    Blue,
}

impl Pixel {
    fn new_rgb(rgb: u32, uses: usize) -> Pixel {
        Pixel {
            rgb,
            r: (rgb & 0xFF) as u8,
            g: ((rgb & 0xFF00) >> 8) as u8,
            b: ((rgb & 0xFF0000) >> 16) as u8,
            uses,
        }
    }

    fn new_colors(r: u8, g: u8, b: u8, uses: usize) -> Pixel {
        Pixel {
            rgb: ((r as u32 & 0xff) << 16) | ((g as u32 & 0xff) << 8) | b as u32 & 0xff,
            r,
            g,
            b,
            uses,
        }
    }

    fn distance2(&self, r: u8, g: u8, b: u8) -> i32 {
        // returns the squared distance between (red, grn, blu)
        // and this this color
        let dr = self.r as i32 - r as i32;
        let dg = self.g as i32 - g as i32;
        let db = self.b as i32 - b as i32;
        return dr * dr + dg * dg + db * db;
    }
}

impl Box {
    fn new(lower: usize, upper: usize, level: usize) -> Box {
        Box {
            lower,
            upper,
            level,

            ..Default::default()
        }
    }

    fn color_count(&self) -> usize {
        self.upper - self.lower
    }

    fn split_box(&mut self, pixels: &mut Vec<Pixel>) -> Option<Box> {
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

    fn get_longest_color_dimension(&mut self, pixels: &Vec<Pixel>) -> ColorDimension {
        let mut min_r = 255;
        let mut max_r = 0;
        let mut min_g = 255;
        let mut max_g = 0;
        let mut min_b = 255;
        let mut max_b = 0;
        let mut count = 0;

        for i in self.lower..self.upper {
            let pixel = pixels[i];
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

    fn find_median(&self, dim: ColorDimension, pixels: &mut Vec<Pixel>) -> usize {
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

    fn get_average_color(&self, pixels: &Vec<Pixel>) -> Pixel {
        let mut r_sum = 0;
        let mut g_sum = 0;
        let mut b_sum = 0;
        let mut n = 0usize;
        for pixel in pixels[self.lower..(self.upper + 1)].iter() {
            let cnt = pixel.uses;
            r_sum += cnt * pixel.r as usize;
            g_sum += cnt * pixel.g as usize;
            b_sum += cnt * pixel.b as usize;
            n += cnt;
        }
        // let nd = n as f64;
        let avg_red = (0.5 + r_sum as f64 / n as f64) as u8;
        let avg_grn = (0.5 + g_sum as f64 / n as f64) as u8;
        let avg_blu = (0.5 + b_sum as f64 / n as f64) as u8;
        Pixel::new_colors(avg_red, avg_grn, avg_blu, n)
    }
}

pub struct MedianCut {
    quant_colors: Vec<Pixel>,
}

impl MedianCut {
    pub fn from_pixels_u8_rgba(pixels: &[u8], k_max: u32) -> MedianCut {
        let pixels = unsafe { ::std::slice::from_raw_parts::<u32>(::std::mem::transmute(&pixels[0]), pixels.len() / 4) };

        MedianCut::from_pixels_u32_rgba(pixels, k_max)
    }

    pub fn from_pixels_u32_rgba(pixels: &[u32], k_max: u32) -> MedianCut {
        let mut m = MedianCut {
            quant_colors: Vec::new(),
        };

        m.quant_colors = m.find_representative_colors(&pixels, k_max);
        m.quant_colors.sort_by(|a, b| b.uses.cmp(&a.uses));

        m
    }

    pub fn get_quantized_colors(&self) -> &Vec<Pixel> {
        &self.quant_colors
    }

    pub fn quantize_image_u8_rgba(&self, pixels: &[u8]) -> &[u8] {
        let new_pixels = unsafe { ::std::slice::from_raw_parts::<u32>(::std::mem::transmute(&pixels[0]), pixels.len() / 4) };

        let quant_pixels = self.quantize_image_u32_rgba(new_pixels.to_vec());
        unsafe { ::std::slice::from_raw_parts::<u8>(::std::mem::transmute(&quant_pixels[0]), quant_pixels.len() * 4) }
    }

    pub fn quantize_image_u32_rgba(&self, pixels: Vec<u32>) -> Vec<u32> {
        let mut quant_pixels = pixels.clone();
        for (i, &pixel) in pixels.iter().enumerate() {
            quant_pixels[i] = self.find_closest_color(pixel).rgb | 0xFF000000;
        }
        quant_pixels
    }

    fn create_histogram(&mut self, pixels: &[u32]) -> HashMap<u32, usize> {
        let n = pixels.len();
        let mut histogram = HashMap::new();
        for i in 0..n {
            let count = histogram.entry(0xFFFFFF & pixels[i]).or_insert(0);
            *count += 1;
        }
        histogram
    }

    fn find_representative_colors(&mut self, pixels: &[u32], k_max: u32) -> Vec<Pixel> {
        // create color histogram
        let histogram = self.create_histogram(pixels);
        let color_num = histogram.len();

        let mut pixels = Vec::with_capacity(color_num);
        for (&rgb, &cnt) in &histogram {
            pixels.push(Pixel::new_rgb(rgb, cnt));
        }

        let r_cols = if color_num <= k_max as usize {
            // image has fewer colors than k_max
            pixels.clone()
        } else {
            let initial_box = Box::new(0, color_num - 1, 0);
            let mut color_set = Vec::new();
            color_set.push(initial_box);
            let mut k = 1;
            let mut done = false;
            while k < k_max && !done {
                let new_box = if let Some(next_box) = self.find_box_to_split(&mut color_set) {
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

            self.average_colors(&color_set, &pixels)
        };
        r_cols
    }

    fn find_closest_color(&self, rgb: u32) -> Pixel {
        let idx = self.find_closest_color_index(rgb);
        self.quant_colors[idx]
    }

    fn find_closest_color_index(&self, rgb: u32) -> usize {
        let r = ((rgb & 0xFF0000) >> 16) as u8;
        let g = ((rgb & 0xFF00) >> 8) as u8;
        let b = (rgb & 0xFF) as u8;
        let mut min_idx = 0;
        let mut min_distance = ::std::i32::MAX;
        for (i, &pixel) in self.quant_colors.iter().enumerate() {
            let d = pixel.distance2(r, g, b);
            if d < min_distance {
                min_distance = d;
                min_idx = i;
            }
        }
        min_idx
    }

    fn average_colors(&mut self, boxees: &Vec<Box>, pixels: &Vec<Pixel>) -> Vec<Pixel> {
        let n = boxees.len();
        let mut avg_colors = Vec::with_capacity(n);
        for b in boxees {
            avg_colors.push(b.get_average_color(&pixels));
        }
        return avg_colors;
    }

    fn find_box_to_split<'a>(&self, boxes: &'a mut Vec<Box>) -> Option<&'a mut Box> {
        let mut box_to_split = None;
        // from the set of splitable color boxes
        // select the one with the minimum level
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