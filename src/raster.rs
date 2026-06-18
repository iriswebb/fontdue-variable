/* Notice to anyone that wants to repurpose the raster for your library:
 * Please don't reuse this raster. Fontdue's raster is very unsafe, with nuanced invariants that
 * need to be accounted for. Fontdue sanitizes the input that the raster will consume to ensure it
 * is safe. Please be aware of this.
 */

use crate::math::Line;
use crate::Glyph;
use alloc::vec;
use alloc::vec::*;
use core::f32::math::*;
use core::simd::f32x4;

pub struct Raster {
    w: usize,
    h: usize,
    a: Vec<f32>,
}

#[inline(always)]
pub fn sub_integer(s: f32x4, other: f32x4) -> f32x4 {
    // Convert to u32 bits, subtract, convert back
    f32x4::from([
        f32::from_bits(s[0].to_bits() - other[0].to_bits()),
        f32::from_bits(s[1].to_bits() - other[1].to_bits()),
        f32::from_bits(s[2].to_bits() - other[2].to_bits()),
        f32::from_bits(s[3].to_bits() - other[3].to_bits()),
    ])
}

#[inline(always)]
pub fn trunc_simd(s: f32x4) -> f32x4 {
    f32x4::from_array([trunc(s[0]), trunc(s[1]), trunc(s[2]), trunc(s[3])])
}

impl Raster {
    pub fn new(w: usize, h: usize) -> Raster {
        Raster {
            w,
            h,
            a: vec![0.0; w * h + 3],
        }
    }

    pub(crate) fn draw(&mut self, glyph: &Glyph, scale_x: f32, scale_y: f32, offset_x: f32, offset_y: f32) {
        let params = f32x4::from_array([1.0 / scale_x, 1.0 / scale_y, scale_x, scale_y]);
        let scale = f32x4::from_array([scale_x, scale_y, scale_x, scale_y]);
        let offset = f32x4::from_array([offset_x, offset_y, offset_x, offset_y]);
        for line in &glyph.v_lines {
            self.v_line(line, line.coords * scale + offset);
        }
        for line in &glyph.m_lines {
            self.m_line(line, line.coords * scale + offset, line.params * params);
        }
    }

    fn add(&mut self, index: usize, height: f32, mid_x: f32) {
        // This is fast and hip.
        unsafe {
            let m = height * mid_x;
            *self.a.get_unchecked_mut(index) += height - m;
            *self.a.get_unchecked_mut(index + 1) += m;
        }

        // This is safe but slow.
        // let m = height * mid_x;
        // self.a[index] += height - m;
        // self.a[index + 1] += m;
    }

    fn v_line(&mut self, line: &Line, coords: f32x4) {
        let [x0, y0, _, y1] = coords.as_array();
        let temp = trunc_simd(sub_integer(coords, line.nudge));
        let [start_x, start_y, end_x, end_y] = temp.as_array();
        let [_, mut target_y, _, _] = (temp + line.adjustment).as_array();
        let sy = 1f32.copysign(y1 - y0);
        let mut y_prev = *y0;
        let mut index = (start_x + start_y * self.w as f32) as i32;
        let index_y_inc = (self.w as f32).copysign(sy) as i32;
        let mut dist = ((start_y - end_y).abs()) as i32;
        let mid_x = fract(*x0);
        while dist > 0 {
            dist -= 1;
            self.add(index as usize, y_prev - target_y, mid_x);
            index += index_y_inc;
            y_prev = target_y;
            target_y += sy;
        }
        self.add((end_x + end_y * self.w as f32) as usize, y_prev - y1, mid_x);
    }

    fn m_line(&mut self, line: &Line, coords: f32x4, params: f32x4) {
        let [x0, y0, x1, y1] = coords.as_array();
        let temp = trunc_simd(sub_integer(coords, line.nudge));
        let [start_x, start_y, end_x, end_y] = temp.as_array();
        let [tdx, tdy, dx, dy] = params.as_array();
        let [mut target_x, mut target_y, _, _] = (temp + line.adjustment).as_array();
        let sx = 1f32.copysign(*tdx);
        let sy = 1f32.copysign(*tdy);
        let mut tmx = tdx * (target_x - x0);
        let mut tmy = tdy * (target_y - y0);
        let tdx = tdx.abs();
        let tdy = tdy.abs();
        let mut x_prev = *x0;
        let mut y_prev = *y0;
        let mut index = (start_x + start_y * self.w as f32) as i32;
        let index_x_inc = (sx) as i32;
        let index_y_inc = ((self.w as f32).copysign(sy)) as i32;
        let mut dist = ((start_x - end_x).abs() + (start_y - end_y).abs()) as i32;
        while dist > 0 {
            dist -= 1;
            let prev_index = index;
            let y_next: f32;
            let x_next: f32;
            if tmx < tmy {
                y_next = tmx * dy + y0; // FMA is not faster.
                x_next = target_x;
                tmx += tdx;
                target_x += sx;
                index += index_x_inc;
            } else {
                y_next = target_y;
                x_next = tmy * dx + x0;
                tmy += tdy;
                target_y += sy;
                index += index_y_inc;
            }
            self.add(prev_index as usize, y_prev - y_next, fract((x_prev + x_next) / 2.0));
            x_prev = x_next;
            y_prev = y_next;
        }
        self.add((end_x + end_y * self.w as f32) as usize, y_prev - y1, fract((x_prev + x1) / 2.0));
    }

    pub fn get_bitmap(&self) -> Vec<u8> {
        let length = self.w * self.h;
        let mut height = 0.0;
        assert!(length <= self.a.len());
        let mut output = vec![0; length];
        for i in 0..length {
            height += self.a[i];
            // Clamping because as u8 is undefined outside of its range in rustc.
            output[i] = (height.abs() * 255.9).clamp(0.0, 255.0) as u8;
        }
        output
    }
}
