mod median_cut;

pub use median_cut::MedianCut;

#[derive(Debug)]
pub struct ColorNode {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub uses: usize,
}

impl ColorNode {
    #[inline]
    pub fn convert_argb_to_u32(a: u8, r: u8, g: u8, b: u8) -> u32 {
        (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32
    }

    #[inline]
    pub fn convert_u32_to_argb(argb: u32) -> [u8;4] {
        let a = ((argb & 0xff000000) >> 24) as u8;
        let r = ((argb & 0xff0000) >> 16) as u8;
        let g = ((argb & 0xff00) >> 8) as u8;
        let b = (argb & 0xff) as u8;
        [a, r, g, b]
    }

    pub fn from_argb(argb: u32, uses: usize) -> ColorNode {
        let [a, r, g, b] = ColorNode::convert_u32_to_argb(argb);
        ColorNode { a, r, g, b, uses, }
    }

    pub fn from_abgr(abgr: u32, uses: usize) -> ColorNode {
        let [a, b, g, r] = ColorNode::convert_u32_to_argb(abgr);
        ColorNode { a, r, g, b, uses, }
    }

    pub fn to_argb(&self) -> u32 {
        ColorNode::convert_argb_to_u32(self.a, self.r, self.g, self.b)
    }

    pub fn to_abgr(&self) -> u32 {
        ColorNode::convert_argb_to_u32(self.a, self.b, self.g, self.r)
    }

    pub fn from_color(a: u8, r: u8, g: u8, b: u8, uses: usize) -> ColorNode {
        ColorNode {
            a,
            r,
            g,
            b,
            uses,
        }
    }

    fn distance2(&self, r: u8, g: u8, b: u8) -> i32 {
        let dr = self.r as i32 - r as i32;
        let dg = self.g as i32 - g as i32;
        let db = self.b as i32 - b as i32;
        return dr * dr + dg * dg + db * db;
    }
}
