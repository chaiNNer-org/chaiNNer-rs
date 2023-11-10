use super::{
    common::{interp1, interp2, interp3, interp5, interp6, interp7, interp8},
    yuv::IntoYuv,
};
use crate::pixel_art::util::write_4x;
use image_core::Image;
use std::ops::{Add, Mul};

// License: GNU Lesser GPL
// Code translated from https://code.google.com/archive/p/hqx/

pub fn hq4x<T>(src: &Image<T>) -> Image<T>
where
    T: Copy + Default + PartialEq + IntoYuv + Add<T, Output = T> + Mul<f32, Output = T>,
{
    let mut result = Image::from_const(src.size().scale(4.0), T::default());

    let width = src.width();
    let height = src.height();
    let src = src.data();

    let dest = result.data_mut();

    let mut w = [T::default(); 10];

    for y in 0..height {
        let y_m1 = y.saturating_sub(1);
        let y_p1 = (y + 1).min(height - 1);

        for x in 0..width {
            let x_m1 = x.saturating_sub(1);
            let x_p1 = (x + 1).min(width - 1);

            // w1 w2 w3
            // w4 w5 w6
            // w7 w8 w9
            w[1] = src[y_m1 * width + x_m1];
            w[2] = src[y_m1 * width + x];
            w[3] = src[y_m1 * width + x_p1];
            w[4] = src[y * width + x_m1];
            w[5] = src[y * width + x];
            w[6] = src[y * width + x_p1];
            w[7] = src[y_p1 * width + x_m1];
            w[8] = src[y_p1 * width + x];
            w[9] = src[y_p1 * width + x_p1];

            write_4x(dest, width, x, y, hq4x_pixel(&w));
        }
    }

    result
}

fn hq4x_pixel<T, Y>(w: &[T; 10]) -> [T; 16]
where
    T: Copy + Default + PartialEq + IntoYuv<Output = Y> + Add<T, Output = T> + Mul<f32, Output = T>,
    Y: PartialEq,
{
    // w1 w2 w3
    // w4 w5 w6
    // w7 w8 w9

    let mut pattern: u8 = 0;
    let mut flag: u8 = 1;

    let yuv1 = w[5].into_yuv();
    for k in 1..=9 {
        if k == 5 {
            continue;
        }

        if w[k] != w[5] {
            let yuv2 = w[k].into_yuv();
            if yuv1 != yuv2 {
                pattern |= flag;
            }
        }
        flag <<= 1;
    }

    let mut r: [T; 16] = Default::default();

    match pattern {
        0 | 1 | 4 | 32 | 128 | 5 | 132 | 160 | 33 | 129 | 36 | 133 | 164 | 161 | 37 | 165 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        2 | 34 | 130 | 162 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        16 | 17 | 48 | 49 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        64 | 65 | 68 | 69 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        8 | 12 | 136 | 140 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        3 | 35 | 131 | 163 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        6 | 38 | 134 | 166 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        20 | 21 | 52 | 53 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        144 | 145 | 176 | 177 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        192 | 193 | 196 | 197 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        96 | 97 | 100 | 101 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        40 | 44 | 168 | 172 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        9 | 13 | 137 | 141 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        18 | 50 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = interp1(w[5], w[3]);
                r[3] = interp8(w[5], w[3]);
                r[6] = interp3(w[5], w[3]);
                r[7] = interp1(w[5], w[3]);
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[6] = w[5];
                r[7] = interp5(w[6], w[5]);
            }
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        80 | 81 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = interp3(w[5], w[9]);
                r[11] = interp1(w[5], w[9]);
                r[14] = interp1(w[5], w[9]);
                r[15] = interp8(w[5], w[9]);
            } else {
                r[10] = w[5];
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
        }
        72 | 76 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = interp1(w[5], w[7]);
                r[9] = interp3(w[5], w[7]);
                r[12] = interp8(w[5], w[7]);
                r[13] = interp1(w[5], w[7]);
            } else {
                r[8] = interp5(w[4], w[5]);
                r[9] = w[5];
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        10 | 138 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = interp8(w[5], w[1]);
                r[1] = interp1(w[5], w[1]);
                r[4] = interp1(w[5], w[1]);
                r[5] = interp3(w[5], w[1]);
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
                r[5] = w[5];
            }
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        66 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        24 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        7 | 39 | 135 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        148 | 149 | 180 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        224 | 228 | 225 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        41 | 169 | 45 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        22 | 54 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = w[5];
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        208 | 209 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
        }
        104 | 108 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        11 | 139 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[5] = w[5];
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        19 | 51 => {
            if w[2].into_yuv() != w[6].into_yuv() {
                r[0] = interp8(w[5], w[4]);
                r[1] = interp3(w[5], w[4]);
                r[2] = interp1(w[5], w[3]);
                r[3] = interp8(w[5], w[3]);
                r[6] = interp3(w[5], w[3]);
                r[7] = interp1(w[5], w[3]);
            } else {
                r[0] = interp1(w[5], w[2]);
                r[1] = interp1(w[2], w[5]);
                r[2] = interp8(w[2], w[6]);
                r[3] = interp5(w[2], w[6]);
                r[6] = interp7(w[5], w[6], w[2]);
                r[7] = interp2(w[6], w[5], w[2]);
            }
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        146 | 178 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = interp1(w[5], w[3]);
                r[3] = interp8(w[5], w[3]);
                r[6] = interp3(w[5], w[3]);
                r[7] = interp1(w[5], w[3]);
                r[11] = interp3(w[5], w[8]);
                r[15] = interp8(w[5], w[8]);
            } else {
                r[2] = interp2(w[2], w[5], w[6]);
                r[3] = interp5(w[2], w[6]);
                r[6] = interp7(w[5], w[6], w[2]);
                r[7] = interp8(w[6], w[2]);
                r[11] = interp1(w[6], w[5]);
                r[15] = interp1(w[5], w[6]);
            }
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp3(w[5], w[8]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp8(w[5], w[8]);
        }
        84 | 85 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp8(w[5], w[2]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r[3] = interp8(w[5], w[2]);
                r[7] = interp3(w[5], w[2]);
                r[10] = interp3(w[5], w[9]);
                r[11] = interp1(w[5], w[9]);
                r[14] = interp1(w[5], w[9]);
                r[15] = interp8(w[5], w[9]);
            } else {
                r[3] = interp1(w[5], w[6]);
                r[7] = interp1(w[6], w[5]);
                r[10] = interp7(w[5], w[6], w[8]);
                r[11] = interp8(w[6], w[8]);
                r[14] = interp2(w[8], w[5], w[6]);
                r[15] = interp5(w[8], w[6]);
            }
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp3(w[5], w[2]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
        }
        112 | 113 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = interp3(w[5], w[9]);
                r[11] = interp1(w[5], w[9]);
                r[12] = interp8(w[5], w[4]);
                r[13] = interp3(w[5], w[4]);
                r[14] = interp1(w[5], w[9]);
                r[15] = interp8(w[5], w[9]);
            } else {
                r[10] = interp7(w[5], w[6], w[8]);
                r[11] = interp2(w[6], w[5], w[8]);
                r[12] = interp1(w[5], w[8]);
                r[13] = interp1(w[8], w[5]);
                r[14] = interp8(w[8], w[6]);
                r[15] = interp5(w[8], w[6]);
            }
        }
        200 | 204 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = interp1(w[5], w[7]);
                r[9] = interp3(w[5], w[7]);
                r[12] = interp8(w[5], w[7]);
                r[13] = interp1(w[5], w[7]);
                r[14] = interp3(w[5], w[6]);
                r[15] = interp8(w[5], w[6]);
            } else {
                r[8] = interp2(w[4], w[5], w[8]);
                r[9] = interp7(w[5], w[4], w[8]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp8(w[8], w[4]);
                r[14] = interp1(w[8], w[5]);
                r[15] = interp1(w[5], w[8]);
            }
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
        }
        73 | 77 => {
            if w[8].into_yuv() != w[4].into_yuv() {
                r[0] = interp8(w[5], w[2]);
                r[4] = interp3(w[5], w[2]);
                r[8] = interp1(w[5], w[7]);
                r[9] = interp3(w[5], w[7]);
                r[12] = interp8(w[5], w[7]);
                r[13] = interp1(w[5], w[7]);
            } else {
                r[0] = interp1(w[5], w[4]);
                r[4] = interp1(w[4], w[5]);
                r[8] = interp8(w[4], w[8]);
                r[9] = interp7(w[5], w[4], w[8]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp2(w[8], w[5], w[4]);
            }
            r[1] = interp8(w[5], w[2]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        42 | 170 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = interp8(w[5], w[1]);
                r[1] = interp1(w[5], w[1]);
                r[4] = interp1(w[5], w[1]);
                r[5] = interp3(w[5], w[1]);
                r[8] = interp3(w[5], w[8]);
                r[12] = interp8(w[5], w[8]);
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp2(w[2], w[5], w[4]);
                r[4] = interp8(w[4], w[2]);
                r[5] = interp7(w[5], w[4], w[2]);
                r[8] = interp1(w[4], w[5]);
                r[12] = interp1(w[5], w[4]);
            }
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        14 | 142 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = interp8(w[5], w[1]);
                r[1] = interp1(w[5], w[1]);
                r[2] = interp3(w[5], w[6]);
                r[3] = interp8(w[5], w[6]);
                r[4] = interp1(w[5], w[1]);
                r[5] = interp3(w[5], w[1]);
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp8(w[2], w[4]);
                r[2] = interp1(w[2], w[5]);
                r[3] = interp1(w[5], w[2]);
                r[4] = interp2(w[4], w[5], w[2]);
                r[5] = interp7(w[5], w[4], w[2]);
            }
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        67 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        70 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        28 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        152 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        194 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        98 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        56 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        25 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        26 | 31 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[5] = w[5];
            r[6] = w[5];
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        82 | 214 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = w[5];
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
        }
        88 | 248 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
        }
        74 | 107 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[5] = w[5];
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        27 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[5] = w[5];
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        86 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = w[5];
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        216 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
        }
        106 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        30 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = w[5];
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        210 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
        }
        120 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        75 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[5] = w[5];
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        29 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        198 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        184 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        99 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        57 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        71 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        156 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        226 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        60 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        195 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        102 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        153 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        58 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = interp8(w[5], w[1]);
                r[1] = interp1(w[5], w[1]);
                r[4] = interp1(w[5], w[1]);
                r[5] = interp3(w[5], w[1]);
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
                r[1] = interp1(w[5], w[2]);
                r[4] = interp1(w[5], w[4]);
                r[5] = w[5];
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = interp1(w[5], w[3]);
                r[3] = interp8(w[5], w[3]);
                r[6] = interp3(w[5], w[3]);
                r[7] = interp1(w[5], w[3]);
            } else {
                r[2] = interp1(w[5], w[2]);
                r[3] = interp2(w[5], w[2], w[6]);
                r[6] = w[5];
                r[7] = interp1(w[5], w[6]);
            }
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        83 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = interp1(w[5], w[3]);
                r[3] = interp8(w[5], w[3]);
                r[6] = interp3(w[5], w[3]);
                r[7] = interp1(w[5], w[3]);
            } else {
                r[2] = interp1(w[5], w[2]);
                r[3] = interp2(w[5], w[2], w[6]);
                r[6] = w[5];
                r[7] = interp1(w[5], w[6]);
            }
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = interp3(w[5], w[9]);
                r[11] = interp1(w[5], w[9]);
                r[14] = interp1(w[5], w[9]);
                r[15] = interp8(w[5], w[9]);
            } else {
                r[10] = w[5];
                r[11] = interp1(w[5], w[6]);
                r[14] = interp1(w[5], w[8]);
                r[15] = interp2(w[5], w[8], w[6]);
            }
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
        }
        92 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = interp1(w[5], w[7]);
                r[9] = interp3(w[5], w[7]);
                r[12] = interp8(w[5], w[7]);
                r[13] = interp1(w[5], w[7]);
            } else {
                r[8] = interp1(w[5], w[4]);
                r[9] = w[5];
                r[12] = interp2(w[5], w[8], w[4]);
                r[13] = interp1(w[5], w[8]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = interp3(w[5], w[9]);
                r[11] = interp1(w[5], w[9]);
                r[14] = interp1(w[5], w[9]);
                r[15] = interp8(w[5], w[9]);
            } else {
                r[10] = w[5];
                r[11] = interp1(w[5], w[6]);
                r[14] = interp1(w[5], w[8]);
                r[15] = interp2(w[5], w[8], w[6]);
            }
        }
        202 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = interp8(w[5], w[1]);
                r[1] = interp1(w[5], w[1]);
                r[4] = interp1(w[5], w[1]);
                r[5] = interp3(w[5], w[1]);
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
                r[1] = interp1(w[5], w[2]);
                r[4] = interp1(w[5], w[4]);
                r[5] = w[5];
            }
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = interp1(w[5], w[7]);
                r[9] = interp3(w[5], w[7]);
                r[12] = interp8(w[5], w[7]);
                r[13] = interp1(w[5], w[7]);
            } else {
                r[8] = interp1(w[5], w[4]);
                r[9] = w[5];
                r[12] = interp2(w[5], w[8], w[4]);
                r[13] = interp1(w[5], w[8]);
            }
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        78 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = interp8(w[5], w[1]);
                r[1] = interp1(w[5], w[1]);
                r[4] = interp1(w[5], w[1]);
                r[5] = interp3(w[5], w[1]);
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
                r[1] = interp1(w[5], w[2]);
                r[4] = interp1(w[5], w[4]);
                r[5] = w[5];
            }
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = interp1(w[5], w[7]);
                r[9] = interp3(w[5], w[7]);
                r[12] = interp8(w[5], w[7]);
                r[13] = interp1(w[5], w[7]);
            } else {
                r[8] = interp1(w[5], w[4]);
                r[9] = w[5];
                r[12] = interp2(w[5], w[8], w[4]);
                r[13] = interp1(w[5], w[8]);
            }
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        154 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = interp8(w[5], w[1]);
                r[1] = interp1(w[5], w[1]);
                r[4] = interp1(w[5], w[1]);
                r[5] = interp3(w[5], w[1]);
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
                r[1] = interp1(w[5], w[2]);
                r[4] = interp1(w[5], w[4]);
                r[5] = w[5];
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = interp1(w[5], w[3]);
                r[3] = interp8(w[5], w[3]);
                r[6] = interp3(w[5], w[3]);
                r[7] = interp1(w[5], w[3]);
            } else {
                r[2] = interp1(w[5], w[2]);
                r[3] = interp2(w[5], w[2], w[6]);
                r[6] = w[5];
                r[7] = interp1(w[5], w[6]);
            }
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        114 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = interp1(w[5], w[3]);
                r[3] = interp8(w[5], w[3]);
                r[6] = interp3(w[5], w[3]);
                r[7] = interp1(w[5], w[3]);
            } else {
                r[2] = interp1(w[5], w[2]);
                r[3] = interp2(w[5], w[2], w[6]);
                r[6] = w[5];
                r[7] = interp1(w[5], w[6]);
            }
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = interp3(w[5], w[9]);
                r[11] = interp1(w[5], w[9]);
                r[14] = interp1(w[5], w[9]);
                r[15] = interp8(w[5], w[9]);
            } else {
                r[10] = w[5];
                r[11] = interp1(w[5], w[6]);
                r[14] = interp1(w[5], w[8]);
                r[15] = interp2(w[5], w[8], w[6]);
            }
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
        }
        89 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = interp1(w[5], w[7]);
                r[9] = interp3(w[5], w[7]);
                r[12] = interp8(w[5], w[7]);
                r[13] = interp1(w[5], w[7]);
            } else {
                r[8] = interp1(w[5], w[4]);
                r[9] = w[5];
                r[12] = interp2(w[5], w[8], w[4]);
                r[13] = interp1(w[5], w[8]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = interp3(w[5], w[9]);
                r[11] = interp1(w[5], w[9]);
                r[14] = interp1(w[5], w[9]);
                r[15] = interp8(w[5], w[9]);
            } else {
                r[10] = w[5];
                r[11] = interp1(w[5], w[6]);
                r[14] = interp1(w[5], w[8]);
                r[15] = interp2(w[5], w[8], w[6]);
            }
        }
        90 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = interp8(w[5], w[1]);
                r[1] = interp1(w[5], w[1]);
                r[4] = interp1(w[5], w[1]);
                r[5] = interp3(w[5], w[1]);
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
                r[1] = interp1(w[5], w[2]);
                r[4] = interp1(w[5], w[4]);
                r[5] = w[5];
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = interp1(w[5], w[3]);
                r[3] = interp8(w[5], w[3]);
                r[6] = interp3(w[5], w[3]);
                r[7] = interp1(w[5], w[3]);
            } else {
                r[2] = interp1(w[5], w[2]);
                r[3] = interp2(w[5], w[2], w[6]);
                r[6] = w[5];
                r[7] = interp1(w[5], w[6]);
            }
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = interp1(w[5], w[7]);
                r[9] = interp3(w[5], w[7]);
                r[12] = interp8(w[5], w[7]);
                r[13] = interp1(w[5], w[7]);
            } else {
                r[8] = interp1(w[5], w[4]);
                r[9] = w[5];
                r[12] = interp2(w[5], w[8], w[4]);
                r[13] = interp1(w[5], w[8]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = interp3(w[5], w[9]);
                r[11] = interp1(w[5], w[9]);
                r[14] = interp1(w[5], w[9]);
                r[15] = interp8(w[5], w[9]);
            } else {
                r[10] = w[5];
                r[11] = interp1(w[5], w[6]);
                r[14] = interp1(w[5], w[8]);
                r[15] = interp2(w[5], w[8], w[6]);
            }
        }
        55 | 23 => {
            if w[2].into_yuv() != w[6].into_yuv() {
                r[0] = interp8(w[5], w[4]);
                r[1] = interp3(w[5], w[4]);
                r[2] = w[5];
                r[3] = w[5];
                r[6] = w[5];
                r[7] = w[5];
            } else {
                r[0] = interp1(w[5], w[2]);
                r[1] = interp1(w[2], w[5]);
                r[2] = interp8(w[2], w[6]);
                r[3] = interp5(w[2], w[6]);
                r[6] = interp7(w[5], w[6], w[2]);
                r[7] = interp2(w[6], w[5], w[2]);
            }
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        182 | 150 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[6] = w[5];
                r[7] = w[5];
                r[11] = interp3(w[5], w[8]);
                r[15] = interp8(w[5], w[8]);
            } else {
                r[2] = interp2(w[2], w[5], w[6]);
                r[3] = interp5(w[2], w[6]);
                r[6] = interp7(w[5], w[6], w[2]);
                r[7] = interp8(w[6], w[2]);
                r[11] = interp1(w[6], w[5]);
                r[15] = interp1(w[5], w[6]);
            }
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp3(w[5], w[8]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp8(w[5], w[8]);
        }
        213 | 212 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp8(w[5], w[2]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r[3] = interp8(w[5], w[2]);
                r[7] = interp3(w[5], w[2]);
                r[10] = w[5];
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[3] = interp1(w[5], w[6]);
                r[7] = interp1(w[6], w[5]);
                r[10] = interp7(w[5], w[6], w[8]);
                r[11] = interp8(w[6], w[8]);
                r[14] = interp2(w[8], w[5], w[6]);
                r[15] = interp5(w[8], w[6]);
            }
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp3(w[5], w[2]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
        }
        241 | 240 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = w[5];
                r[11] = w[5];
                r[12] = interp8(w[5], w[4]);
                r[13] = interp3(w[5], w[4]);
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[10] = interp7(w[5], w[6], w[8]);
                r[11] = interp2(w[6], w[5], w[8]);
                r[12] = interp1(w[5], w[8]);
                r[13] = interp1(w[8], w[5]);
                r[14] = interp8(w[8], w[6]);
                r[15] = interp5(w[8], w[6]);
            }
        }
        236 | 232 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[9] = w[5];
                r[12] = w[5];
                r[13] = w[5];
                r[14] = interp3(w[5], w[6]);
                r[15] = interp8(w[5], w[6]);
            } else {
                r[8] = interp2(w[4], w[5], w[8]);
                r[9] = interp7(w[5], w[4], w[8]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp8(w[8], w[4]);
                r[14] = interp1(w[8], w[5]);
                r[15] = interp1(w[5], w[8]);
            }
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
        }
        109 | 105 => {
            if w[8].into_yuv() != w[4].into_yuv() {
                r[0] = interp8(w[5], w[2]);
                r[4] = interp3(w[5], w[2]);
                r[8] = w[5];
                r[9] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[0] = interp1(w[5], w[4]);
                r[4] = interp1(w[4], w[5]);
                r[8] = interp8(w[4], w[8]);
                r[9] = interp7(w[5], w[4], w[8]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp2(w[8], w[5], w[4]);
            }
            r[1] = interp8(w[5], w[2]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        171 | 43 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
                r[5] = w[5];
                r[8] = interp3(w[5], w[8]);
                r[12] = interp8(w[5], w[8]);
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp2(w[2], w[5], w[4]);
                r[4] = interp8(w[4], w[2]);
                r[5] = interp7(w[5], w[4], w[2]);
                r[8] = interp1(w[4], w[5]);
                r[12] = interp1(w[5], w[4]);
            }
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        143 | 15 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[2] = interp3(w[5], w[6]);
                r[3] = interp8(w[5], w[6]);
                r[4] = w[5];
                r[5] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp8(w[2], w[4]);
                r[2] = interp1(w[2], w[5]);
                r[3] = interp1(w[5], w[2]);
                r[4] = interp2(w[4], w[5], w[2]);
                r[5] = interp7(w[5], w[4], w[2]);
            }
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        124 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        203 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[5] = w[5];
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        62 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = w[5];
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        211 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
        }
        118 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = w[5];
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        217 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
        }
        110 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        155 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[5] = w[5];
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        188 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        185 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        61 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        157 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        103 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        227 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        230 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        199 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        220 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = interp1(w[5], w[7]);
                r[9] = interp3(w[5], w[7]);
                r[12] = interp8(w[5], w[7]);
                r[13] = interp1(w[5], w[7]);
            } else {
                r[8] = interp1(w[5], w[4]);
                r[9] = w[5];
                r[12] = interp2(w[5], w[8], w[4]);
                r[13] = interp1(w[5], w[8]);
            }
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
        }
        158 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = interp8(w[5], w[1]);
                r[1] = interp1(w[5], w[1]);
                r[4] = interp1(w[5], w[1]);
                r[5] = interp3(w[5], w[1]);
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
                r[1] = interp1(w[5], w[2]);
                r[4] = interp1(w[5], w[4]);
                r[5] = w[5];
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[6] = w[5];
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        234 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = interp8(w[5], w[1]);
                r[1] = interp1(w[5], w[1]);
                r[4] = interp1(w[5], w[1]);
                r[5] = interp3(w[5], w[1]);
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
                r[1] = interp1(w[5], w[2]);
                r[4] = interp1(w[5], w[4]);
                r[5] = w[5];
            }
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        242 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = interp1(w[5], w[3]);
                r[3] = interp8(w[5], w[3]);
                r[6] = interp3(w[5], w[3]);
                r[7] = interp1(w[5], w[3]);
            } else {
                r[2] = interp1(w[5], w[2]);
                r[3] = interp2(w[5], w[2], w[6]);
                r[6] = w[5];
                r[7] = interp1(w[5], w[6]);
            }
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
        }
        59 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = interp1(w[5], w[3]);
                r[3] = interp8(w[5], w[3]);
                r[6] = interp3(w[5], w[3]);
                r[7] = interp1(w[5], w[3]);
            } else {
                r[2] = interp1(w[5], w[2]);
                r[3] = interp2(w[5], w[2], w[6]);
                r[6] = w[5];
                r[7] = interp1(w[5], w[6]);
            }
            r[5] = w[5];
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        121 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = interp3(w[5], w[9]);
                r[11] = interp1(w[5], w[9]);
                r[14] = interp1(w[5], w[9]);
                r[15] = interp8(w[5], w[9]);
            } else {
                r[10] = w[5];
                r[11] = interp1(w[5], w[6]);
                r[14] = interp1(w[5], w[8]);
                r[15] = interp2(w[5], w[8], w[6]);
            }
        }
        87 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = w[5];
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = interp3(w[5], w[9]);
                r[11] = interp1(w[5], w[9]);
                r[14] = interp1(w[5], w[9]);
                r[15] = interp8(w[5], w[9]);
            } else {
                r[10] = w[5];
                r[11] = interp1(w[5], w[6]);
                r[14] = interp1(w[5], w[8]);
                r[15] = interp2(w[5], w[8], w[6]);
            }
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
        }
        79 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[5] = w[5];
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = interp1(w[5], w[7]);
                r[9] = interp3(w[5], w[7]);
                r[12] = interp8(w[5], w[7]);
                r[13] = interp1(w[5], w[7]);
            } else {
                r[8] = interp1(w[5], w[4]);
                r[9] = w[5];
                r[12] = interp2(w[5], w[8], w[4]);
                r[13] = interp1(w[5], w[8]);
            }
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        122 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = interp8(w[5], w[1]);
                r[1] = interp1(w[5], w[1]);
                r[4] = interp1(w[5], w[1]);
                r[5] = interp3(w[5], w[1]);
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
                r[1] = interp1(w[5], w[2]);
                r[4] = interp1(w[5], w[4]);
                r[5] = w[5];
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = interp1(w[5], w[3]);
                r[3] = interp8(w[5], w[3]);
                r[6] = interp3(w[5], w[3]);
                r[7] = interp1(w[5], w[3]);
            } else {
                r[2] = interp1(w[5], w[2]);
                r[3] = interp2(w[5], w[2], w[6]);
                r[6] = w[5];
                r[7] = interp1(w[5], w[6]);
            }
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = interp3(w[5], w[9]);
                r[11] = interp1(w[5], w[9]);
                r[14] = interp1(w[5], w[9]);
                r[15] = interp8(w[5], w[9]);
            } else {
                r[10] = w[5];
                r[11] = interp1(w[5], w[6]);
                r[14] = interp1(w[5], w[8]);
                r[15] = interp2(w[5], w[8], w[6]);
            }
        }
        94 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = interp8(w[5], w[1]);
                r[1] = interp1(w[5], w[1]);
                r[4] = interp1(w[5], w[1]);
                r[5] = interp3(w[5], w[1]);
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
                r[1] = interp1(w[5], w[2]);
                r[4] = interp1(w[5], w[4]);
                r[5] = w[5];
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[6] = w[5];
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = interp1(w[5], w[7]);
                r[9] = interp3(w[5], w[7]);
                r[12] = interp8(w[5], w[7]);
                r[13] = interp1(w[5], w[7]);
            } else {
                r[8] = interp1(w[5], w[4]);
                r[9] = w[5];
                r[12] = interp2(w[5], w[8], w[4]);
                r[13] = interp1(w[5], w[8]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = interp3(w[5], w[9]);
                r[11] = interp1(w[5], w[9]);
                r[14] = interp1(w[5], w[9]);
                r[15] = interp8(w[5], w[9]);
            } else {
                r[10] = w[5];
                r[11] = interp1(w[5], w[6]);
                r[14] = interp1(w[5], w[8]);
                r[15] = interp2(w[5], w[8], w[6]);
            }
        }
        218 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = interp8(w[5], w[1]);
                r[1] = interp1(w[5], w[1]);
                r[4] = interp1(w[5], w[1]);
                r[5] = interp3(w[5], w[1]);
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
                r[1] = interp1(w[5], w[2]);
                r[4] = interp1(w[5], w[4]);
                r[5] = w[5];
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = interp1(w[5], w[3]);
                r[3] = interp8(w[5], w[3]);
                r[6] = interp3(w[5], w[3]);
                r[7] = interp1(w[5], w[3]);
            } else {
                r[2] = interp1(w[5], w[2]);
                r[3] = interp2(w[5], w[2], w[6]);
                r[6] = w[5];
                r[7] = interp1(w[5], w[6]);
            }
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = interp1(w[5], w[7]);
                r[9] = interp3(w[5], w[7]);
                r[12] = interp8(w[5], w[7]);
                r[13] = interp1(w[5], w[7]);
            } else {
                r[8] = interp1(w[5], w[4]);
                r[9] = w[5];
                r[12] = interp2(w[5], w[8], w[4]);
                r[13] = interp1(w[5], w[8]);
            }
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
        }
        91 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = interp1(w[5], w[3]);
                r[3] = interp8(w[5], w[3]);
                r[6] = interp3(w[5], w[3]);
                r[7] = interp1(w[5], w[3]);
            } else {
                r[2] = interp1(w[5], w[2]);
                r[3] = interp2(w[5], w[2], w[6]);
                r[6] = w[5];
                r[7] = interp1(w[5], w[6]);
            }
            r[5] = w[5];
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = interp1(w[5], w[7]);
                r[9] = interp3(w[5], w[7]);
                r[12] = interp8(w[5], w[7]);
                r[13] = interp1(w[5], w[7]);
            } else {
                r[8] = interp1(w[5], w[4]);
                r[9] = w[5];
                r[12] = interp2(w[5], w[8], w[4]);
                r[13] = interp1(w[5], w[8]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = interp3(w[5], w[9]);
                r[11] = interp1(w[5], w[9]);
                r[14] = interp1(w[5], w[9]);
                r[15] = interp8(w[5], w[9]);
            } else {
                r[10] = w[5];
                r[11] = interp1(w[5], w[6]);
                r[14] = interp1(w[5], w[8]);
                r[15] = interp2(w[5], w[8], w[6]);
            }
        }
        229 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        167 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        173 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        181 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        186 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = interp8(w[5], w[1]);
                r[1] = interp1(w[5], w[1]);
                r[4] = interp1(w[5], w[1]);
                r[5] = interp3(w[5], w[1]);
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
                r[1] = interp1(w[5], w[2]);
                r[4] = interp1(w[5], w[4]);
                r[5] = w[5];
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = interp1(w[5], w[3]);
                r[3] = interp8(w[5], w[3]);
                r[6] = interp3(w[5], w[3]);
                r[7] = interp1(w[5], w[3]);
            } else {
                r[2] = interp1(w[5], w[2]);
                r[3] = interp2(w[5], w[2], w[6]);
                r[6] = w[5];
                r[7] = interp1(w[5], w[6]);
            }
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        115 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = interp1(w[5], w[3]);
                r[3] = interp8(w[5], w[3]);
                r[6] = interp3(w[5], w[3]);
                r[7] = interp1(w[5], w[3]);
            } else {
                r[2] = interp1(w[5], w[2]);
                r[3] = interp2(w[5], w[2], w[6]);
                r[6] = w[5];
                r[7] = interp1(w[5], w[6]);
            }
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = interp3(w[5], w[9]);
                r[11] = interp1(w[5], w[9]);
                r[14] = interp1(w[5], w[9]);
                r[15] = interp8(w[5], w[9]);
            } else {
                r[10] = w[5];
                r[11] = interp1(w[5], w[6]);
                r[14] = interp1(w[5], w[8]);
                r[15] = interp2(w[5], w[8], w[6]);
            }
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
        }
        93 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = interp1(w[5], w[7]);
                r[9] = interp3(w[5], w[7]);
                r[12] = interp8(w[5], w[7]);
                r[13] = interp1(w[5], w[7]);
            } else {
                r[8] = interp1(w[5], w[4]);
                r[9] = w[5];
                r[12] = interp2(w[5], w[8], w[4]);
                r[13] = interp1(w[5], w[8]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = interp3(w[5], w[9]);
                r[11] = interp1(w[5], w[9]);
                r[14] = interp1(w[5], w[9]);
                r[15] = interp8(w[5], w[9]);
            } else {
                r[10] = w[5];
                r[11] = interp1(w[5], w[6]);
                r[14] = interp1(w[5], w[8]);
                r[15] = interp2(w[5], w[8], w[6]);
            }
        }
        206 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = interp8(w[5], w[1]);
                r[1] = interp1(w[5], w[1]);
                r[4] = interp1(w[5], w[1]);
                r[5] = interp3(w[5], w[1]);
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
                r[1] = interp1(w[5], w[2]);
                r[4] = interp1(w[5], w[4]);
                r[5] = w[5];
            }
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = interp1(w[5], w[7]);
                r[9] = interp3(w[5], w[7]);
                r[12] = interp8(w[5], w[7]);
                r[13] = interp1(w[5], w[7]);
            } else {
                r[8] = interp1(w[5], w[4]);
                r[9] = w[5];
                r[12] = interp2(w[5], w[8], w[4]);
                r[13] = interp1(w[5], w[8]);
            }
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        205 | 201 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = interp1(w[5], w[7]);
                r[9] = interp3(w[5], w[7]);
                r[12] = interp8(w[5], w[7]);
                r[13] = interp1(w[5], w[7]);
            } else {
                r[8] = interp1(w[5], w[4]);
                r[9] = w[5];
                r[12] = interp2(w[5], w[8], w[4]);
                r[13] = interp1(w[5], w[8]);
            }
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        174 | 46 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = interp8(w[5], w[1]);
                r[1] = interp1(w[5], w[1]);
                r[4] = interp1(w[5], w[1]);
                r[5] = interp3(w[5], w[1]);
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
                r[1] = interp1(w[5], w[2]);
                r[4] = interp1(w[5], w[4]);
                r[5] = w[5];
            }
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        179 | 147 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = interp1(w[5], w[3]);
                r[3] = interp8(w[5], w[3]);
                r[6] = interp3(w[5], w[3]);
                r[7] = interp1(w[5], w[3]);
            } else {
                r[2] = interp1(w[5], w[2]);
                r[3] = interp2(w[5], w[2], w[6]);
                r[6] = w[5];
                r[7] = interp1(w[5], w[6]);
            }
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        117 | 116 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = interp3(w[5], w[9]);
                r[11] = interp1(w[5], w[9]);
                r[14] = interp1(w[5], w[9]);
                r[15] = interp8(w[5], w[9]);
            } else {
                r[10] = w[5];
                r[11] = interp1(w[5], w[6]);
                r[14] = interp1(w[5], w[8]);
                r[15] = interp2(w[5], w[8], w[6]);
            }
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
        }
        189 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        231 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        126 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = w[5];
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        219 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[5] = w[5];
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
        }
        125 => {
            if w[8].into_yuv() != w[4].into_yuv() {
                r[0] = interp8(w[5], w[2]);
                r[4] = interp3(w[5], w[2]);
                r[8] = w[5];
                r[9] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[0] = interp1(w[5], w[4]);
                r[4] = interp1(w[4], w[5]);
                r[8] = interp8(w[4], w[8]);
                r[9] = interp7(w[5], w[4], w[8]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp2(w[8], w[5], w[4]);
            }
            r[1] = interp8(w[5], w[2]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        221 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp8(w[5], w[2]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r[3] = interp8(w[5], w[2]);
                r[7] = interp3(w[5], w[2]);
                r[10] = w[5];
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[3] = interp1(w[5], w[6]);
                r[7] = interp1(w[6], w[5]);
                r[10] = interp7(w[5], w[6], w[8]);
                r[11] = interp8(w[6], w[8]);
                r[14] = interp2(w[8], w[5], w[6]);
                r[15] = interp5(w[8], w[6]);
            }
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp3(w[5], w[2]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
        }
        207 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[2] = interp3(w[5], w[6]);
                r[3] = interp8(w[5], w[6]);
                r[4] = w[5];
                r[5] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp8(w[2], w[4]);
                r[2] = interp1(w[2], w[5]);
                r[3] = interp1(w[5], w[2]);
                r[4] = interp2(w[4], w[5], w[2]);
                r[5] = interp7(w[5], w[4], w[2]);
            }
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        238 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[9] = w[5];
                r[12] = w[5];
                r[13] = w[5];
                r[14] = interp3(w[5], w[6]);
                r[15] = interp8(w[5], w[6]);
            } else {
                r[8] = interp2(w[4], w[5], w[8]);
                r[9] = interp7(w[5], w[4], w[8]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp8(w[8], w[4]);
                r[14] = interp1(w[8], w[5]);
                r[15] = interp1(w[5], w[8]);
            }
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
        }
        190 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[6] = w[5];
                r[7] = w[5];
                r[11] = interp3(w[5], w[8]);
                r[15] = interp8(w[5], w[8]);
            } else {
                r[2] = interp2(w[2], w[5], w[6]);
                r[3] = interp5(w[2], w[6]);
                r[6] = interp7(w[5], w[6], w[2]);
                r[7] = interp8(w[6], w[2]);
                r[11] = interp1(w[6], w[5]);
                r[15] = interp1(w[5], w[6]);
            }
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp3(w[5], w[8]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp8(w[5], w[8]);
        }
        187 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
                r[5] = w[5];
                r[8] = interp3(w[5], w[8]);
                r[12] = interp8(w[5], w[8]);
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp2(w[2], w[5], w[4]);
                r[4] = interp8(w[4], w[2]);
                r[5] = interp7(w[5], w[4], w[2]);
                r[8] = interp1(w[4], w[5]);
                r[12] = interp1(w[5], w[4]);
            }
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        243 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r[10] = w[5];
                r[11] = w[5];
                r[12] = interp8(w[5], w[4]);
                r[13] = interp3(w[5], w[4]);
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[10] = interp7(w[5], w[6], w[8]);
                r[11] = interp2(w[6], w[5], w[8]);
                r[12] = interp1(w[5], w[8]);
                r[13] = interp1(w[8], w[5]);
                r[14] = interp8(w[8], w[6]);
                r[15] = interp5(w[8], w[6]);
            }
        }
        119 => {
            if w[2].into_yuv() != w[6].into_yuv() {
                r[0] = interp8(w[5], w[4]);
                r[1] = interp3(w[5], w[4]);
                r[2] = w[5];
                r[3] = w[5];
                r[6] = w[5];
                r[7] = w[5];
            } else {
                r[0] = interp1(w[5], w[2]);
                r[1] = interp1(w[2], w[5]);
                r[2] = interp8(w[2], w[6]);
                r[3] = interp5(w[2], w[6]);
                r[6] = interp7(w[5], w[6], w[2]);
                r[7] = interp2(w[6], w[5], w[2]);
            }
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        237 | 233 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp6(w[5], w[2], w[6]);
            r[3] = interp2(w[5], w[2], w[6]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp7(w[5], w[6], w[2]);
            r[7] = interp6(w[5], w[6], w[2]);
            r[8] = w[5];
            r[9] = w[5];
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[12] = w[5];
            } else {
                r[12] = interp2(w[5], w[8], w[4]);
            }
            r[13] = w[5];
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        175 | 47 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
            }
            r[1] = w[5];
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[4] = w[5];
            r[5] = w[5];
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp7(w[5], w[6], w[8]);
            r[11] = interp6(w[5], w[6], w[8]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp6(w[5], w[8], w[6]);
            r[15] = interp2(w[5], w[8], w[6]);
        }
        183 | 151 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            r[2] = w[5];
            if w[2].into_yuv() != w[6].into_yuv() {
                r[3] = w[5];
            } else {
                r[3] = interp2(w[5], w[2], w[6]);
            }
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = w[5];
            r[7] = w[5];
            r[8] = interp6(w[5], w[4], w[8]);
            r[9] = interp7(w[5], w[4], w[8]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp2(w[5], w[8], w[4]);
            r[13] = interp6(w[5], w[8], w[4]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        245 | 244 => {
            r[0] = interp2(w[5], w[2], w[4]);
            r[1] = interp6(w[5], w[2], w[4]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp6(w[5], w[4], w[2]);
            r[5] = interp7(w[5], w[4], w[2]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = w[5];
            r[11] = w[5];
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
            r[14] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[15] = w[5];
            } else {
                r[15] = interp2(w[5], w[8], w[6]);
            }
        }
        250 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
        }
        123 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[5] = w[5];
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        95 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[5] = w[5];
            r[6] = w[5];
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        222 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = w[5];
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
        }
        252 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp6(w[5], w[2], w[1]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            r[10] = w[5];
            r[11] = w[5];
            r[14] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[15] = w[5];
            } else {
                r[15] = interp2(w[5], w[8], w[6]);
            }
        }
        249 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp6(w[5], w[2], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = w[5];
            r[9] = w[5];
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
            if w[8].into_yuv() != w[4].into_yuv() {
                r[12] = w[5];
            } else {
                r[12] = interp2(w[5], w[8], w[4]);
            }
            r[13] = w[5];
        }
        235 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[5] = w[5];
            r[6] = interp3(w[5], w[3]);
            r[7] = interp6(w[5], w[6], w[3]);
            r[8] = w[5];
            r[9] = w[5];
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[12] = w[5];
            } else {
                r[12] = interp2(w[5], w[8], w[4]);
            }
            r[13] = w[5];
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        111 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
            }
            r[1] = w[5];
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[4] = w[5];
            r[5] = w[5];
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            r[10] = interp3(w[5], w[9]);
            r[11] = interp6(w[5], w[6], w[9]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        63 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
            }
            r[1] = w[5];
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[4] = w[5];
            r[5] = w[5];
            r[6] = w[5];
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp6(w[5], w[8], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        159 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            r[2] = w[5];
            if w[2].into_yuv() != w[6].into_yuv() {
                r[3] = w[5];
            } else {
                r[3] = interp2(w[5], w[2], w[6]);
            }
            r[5] = w[5];
            r[6] = w[5];
            r[7] = w[5];
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp8(w[5], w[7]);
            r[13] = interp6(w[5], w[8], w[7]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        215 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            r[2] = w[5];
            if w[2].into_yuv() != w[6].into_yuv() {
                r[3] = w[5];
            } else {
                r[3] = interp2(w[5], w[2], w[6]);
            }
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = w[5];
            r[7] = w[5];
            r[8] = interp6(w[5], w[4], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
        }
        246 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[4] = interp6(w[5], w[4], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = w[5];
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = w[5];
            r[11] = w[5];
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
            r[14] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[15] = w[5];
            } else {
                r[15] = interp2(w[5], w[8], w[6]);
            }
        }
        254 => {
            r[0] = interp8(w[5], w[1]);
            r[1] = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[4] = interp1(w[5], w[1]);
            r[5] = interp3(w[5], w[1]);
            r[6] = w[5];
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            r[10] = w[5];
            r[11] = w[5];
            r[14] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[15] = w[5];
            } else {
                r[15] = interp2(w[5], w[8], w[6]);
            }
        }
        253 => {
            r[0] = interp8(w[5], w[2]);
            r[1] = interp8(w[5], w[2]);
            r[2] = interp8(w[5], w[2]);
            r[3] = interp8(w[5], w[2]);
            r[4] = interp3(w[5], w[2]);
            r[5] = interp3(w[5], w[2]);
            r[6] = interp3(w[5], w[2]);
            r[7] = interp3(w[5], w[2]);
            r[8] = w[5];
            r[9] = w[5];
            r[10] = w[5];
            r[11] = w[5];
            if w[8].into_yuv() != w[4].into_yuv() {
                r[12] = w[5];
            } else {
                r[12] = interp2(w[5], w[8], w[4]);
            }
            r[13] = w[5];
            r[14] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[15] = w[5];
            } else {
                r[15] = interp2(w[5], w[8], w[6]);
            }
        }
        251 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            r[2] = interp1(w[5], w[3]);
            r[3] = interp8(w[5], w[3]);
            r[5] = w[5];
            r[6] = interp3(w[5], w[3]);
            r[7] = interp1(w[5], w[3]);
            r[8] = w[5];
            r[9] = w[5];
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
            if w[8].into_yuv() != w[4].into_yuv() {
                r[12] = w[5];
            } else {
                r[12] = interp2(w[5], w[8], w[4]);
            }
            r[13] = w[5];
        }
        239 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
            }
            r[1] = w[5];
            r[2] = interp3(w[5], w[6]);
            r[3] = interp8(w[5], w[6]);
            r[4] = w[5];
            r[5] = w[5];
            r[6] = interp3(w[5], w[6]);
            r[7] = interp8(w[5], w[6]);
            r[8] = w[5];
            r[9] = w[5];
            r[10] = interp3(w[5], w[6]);
            r[11] = interp8(w[5], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r[12] = w[5];
            } else {
                r[12] = interp2(w[5], w[8], w[4]);
            }
            r[13] = w[5];
            r[14] = interp3(w[5], w[6]);
            r[15] = interp8(w[5], w[6]);
        }
        127 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
            }
            r[1] = w[5];
            if w[2].into_yuv() != w[6].into_yuv() {
                r[2] = w[5];
                r[3] = w[5];
                r[7] = w[5];
            } else {
                r[2] = interp5(w[2], w[5]);
                r[3] = interp5(w[2], w[6]);
                r[7] = interp5(w[6], w[5]);
            }
            r[4] = w[5];
            r[5] = w[5];
            r[6] = w[5];
            if w[8].into_yuv() != w[4].into_yuv() {
                r[8] = w[5];
                r[12] = w[5];
                r[13] = w[5];
            } else {
                r[8] = interp5(w[4], w[5]);
                r[12] = interp5(w[8], w[4]);
                r[13] = interp5(w[8], w[5]);
            }
            r[9] = w[5];
            r[10] = interp3(w[5], w[9]);
            r[11] = interp1(w[5], w[9]);
            r[14] = interp1(w[5], w[9]);
            r[15] = interp8(w[5], w[9]);
        }
        191 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
            }
            r[1] = w[5];
            r[2] = w[5];
            if w[2].into_yuv() != w[6].into_yuv() {
                r[3] = w[5];
            } else {
                r[3] = interp2(w[5], w[2], w[6]);
            }
            r[4] = w[5];
            r[5] = w[5];
            r[6] = w[5];
            r[7] = w[5];
            r[8] = interp3(w[5], w[8]);
            r[9] = interp3(w[5], w[8]);
            r[10] = interp3(w[5], w[8]);
            r[11] = interp3(w[5], w[8]);
            r[12] = interp8(w[5], w[8]);
            r[13] = interp8(w[5], w[8]);
            r[14] = interp8(w[5], w[8]);
            r[15] = interp8(w[5], w[8]);
        }
        223 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
                r[1] = w[5];
                r[4] = w[5];
            } else {
                r[0] = interp5(w[2], w[4]);
                r[1] = interp5(w[2], w[5]);
                r[4] = interp5(w[4], w[5]);
            }
            r[2] = w[5];
            if w[2].into_yuv() != w[6].into_yuv() {
                r[3] = w[5];
            } else {
                r[3] = interp2(w[5], w[2], w[6]);
            }
            r[5] = w[5];
            r[6] = w[5];
            r[7] = w[5];
            r[8] = interp1(w[5], w[7]);
            r[9] = interp3(w[5], w[7]);
            r[10] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[11] = w[5];
                r[14] = w[5];
                r[15] = w[5];
            } else {
                r[11] = interp5(w[6], w[5]);
                r[14] = interp5(w[8], w[5]);
                r[15] = interp5(w[8], w[6]);
            }
            r[12] = interp8(w[5], w[7]);
            r[13] = interp1(w[5], w[7]);
        }
        247 => {
            r[0] = interp8(w[5], w[4]);
            r[1] = interp3(w[5], w[4]);
            r[2] = w[5];
            if w[2].into_yuv() != w[6].into_yuv() {
                r[3] = w[5];
            } else {
                r[3] = interp2(w[5], w[2], w[6]);
            }
            r[4] = interp8(w[5], w[4]);
            r[5] = interp3(w[5], w[4]);
            r[6] = w[5];
            r[7] = w[5];
            r[8] = interp8(w[5], w[4]);
            r[9] = interp3(w[5], w[4]);
            r[10] = w[5];
            r[11] = w[5];
            r[12] = interp8(w[5], w[4]);
            r[13] = interp3(w[5], w[4]);
            r[14] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[15] = w[5];
            } else {
                r[15] = interp2(w[5], w[8], w[6]);
            }
        }
        255 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r[0] = w[5];
            } else {
                r[0] = interp2(w[5], w[2], w[4]);
            }
            r[1] = w[5];
            r[2] = w[5];
            if w[2].into_yuv() != w[6].into_yuv() {
                r[3] = w[5];
            } else {
                r[3] = interp2(w[5], w[2], w[6]);
            }
            r[4] = w[5];
            r[5] = w[5];
            r[6] = w[5];
            r[7] = w[5];
            r[8] = w[5];
            r[9] = w[5];
            r[10] = w[5];
            r[11] = w[5];
            if w[8].into_yuv() != w[4].into_yuv() {
                r[12] = w[5];
            } else {
                r[12] = interp2(w[5], w[8], w[4]);
            }
            r[13] = w[5];
            r[14] = w[5];
            if w[6].into_yuv() != w[8].into_yuv() {
                r[15] = w[5];
            } else {
                r[15] = interp2(w[5], w[8], w[6]);
            }
        }
    }

    r
}

#[cfg(test)]
mod tests {
    use test_util::{data::read_nes_smb, snap::ImageSnapshot};

    #[test]
    fn hq4x() {
        let original = read_nes_smb();

        super::hq4x(&original).snapshot("px_up_hq4x");
    }
}
