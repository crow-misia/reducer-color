use std::cmp::{max, min};

#[derive(Debug)]
pub struct ErrorDiffusion {
    index: usize,
    error_row1: Vec<i32>,
    error_row2: Vec<i32>,
    error_row3: Vec<i32>,
}

impl ErrorDiffusion {
    pub fn new(width: u32) -> ErrorDiffusion {
        let new_width = (width + 4) as usize;
        let mut buf1 = Vec::with_capacity(new_width);
        let mut buf2 = Vec::with_capacity(new_width);
        let mut buf3 = Vec::with_capacity(new_width);
        unsafe { buf1.set_len(new_width) };
        unsafe { buf2.set_len(new_width) };
        unsafe { buf3.set_len(new_width) };
        buf1.fill(0);
        buf2.fill(0);
        buf3.fill(0);
        ErrorDiffusion {
            index: 2,
            error_row1: buf1,
            error_row2: buf2,
            error_row3: buf3,
        }
    }

    #[inline]
    pub fn adjust(&mut self, color: u8) -> u8 {
        let mut index = self.index;
        let stored_error = (self.error_row1).get(index).unwrap();
        let adjust_color = color as i32 + (stored_error >> 5);
        (max(0x00, min(0xff,adjust_color)) & 0xff) as u8
    }

    #[inline]
    pub fn calculate(&mut self, org_color: u8, fixed_color: u8) {
        let mut index = self.index;

        let error_fraction = (org_color as i32) - (fixed_color as i32);
        let error_fraction2 = error_fraction << 1;
        let error_fraction4 = error_fraction << 2;
        let error_fraction8 = error_fraction << 3;

        self.error_row1[index+1] += error_fraction8;
        self.error_row1[index+2] += error_fraction4;

        self.error_row2[index-2] += error_fraction2;
        self.error_row2[index-1] += error_fraction4;
        self.error_row2[index] += error_fraction8;
        self.error_row2[index+1] += error_fraction4;
        self.error_row2[index+2] += error_fraction2;

        self.error_row3[index-2] += error_fraction;
        self.error_row3[index-1] += error_fraction2;
        self.error_row3[index] += error_fraction4;
        self.error_row3[index+1] += error_fraction2;
        self.error_row3[index+2] += error_fraction;

        self.index += 1;
    }

    #[inline]
    pub fn next_row(&mut self) {
        self.index = 2;
        std::mem::swap(&mut self.error_row1, &mut self.error_row3);
        std::mem::swap(&mut self.error_row2, &mut self.error_row3);
        self.error_row3.fill(0);
    }
}
