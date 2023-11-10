use super::{
    common::{interp1, interp10, interp2, interp6, interp7, interp9},
    yuv::IntoYuv,
};
use crate::pixel_art::util::write_2x;
use image_core::Image;
use std::ops::{Add, Mul};

// License: GNU Lesser GPL
// Code translated from https://code.google.com/archive/p/hqx/

pub fn hq2x<T>(src: &Image<T>) -> Image<T>
where
    T: Copy + Default + PartialEq + IntoYuv + Add<T, Output = T> + Mul<f32, Output = T>,
{
    let mut result = Image::from_const(src.size().scale(2.0), T::default());

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

            write_2x(dest, width, x, y, hq2x_pixel(&w));
        }
    }

    result
}

fn hq2x_pixel<T>(w: &[T; 10]) -> [T; 4]
where
    T: Copy + Default + PartialEq + IntoYuv + Add<T, Output = T> + Mul<f32, Output = T>,
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

    let r1: T;
    let r2: T;
    let r3: T;
    let r4: T;

    match pattern {
        0 | 1 | 4 | 32 | 128 | 5 | 132 | 160 | 33 | 129 | 36 | 133 | 164 | 161 | 37 | 165 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        2 | 34 | 130 | 162 => {
            r1 = interp2(w[5], w[1], w[4]);
            r2 = interp2(w[5], w[3], w[6]);
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        16 | 17 | 48 | 49 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        64 | 65 | 68 | 69 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            r3 = interp2(w[5], w[7], w[4]);
            r4 = interp2(w[5], w[9], w[6]);
        }
        8 | 12 | 136 | 140 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        3 | 35 | 131 | 163 => {
            r1 = interp1(w[5], w[4]);
            r2 = interp2(w[5], w[3], w[6]);
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        6 | 38 | 134 | 166 => {
            r1 = interp2(w[5], w[1], w[4]);
            r2 = interp1(w[5], w[6]);
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        20 | 21 | 52 | 53 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp1(w[5], w[2]);
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        144 | 145 | 176 | 177 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp1(w[5], w[8]);
        }
        192 | 193 | 196 | 197 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            r3 = interp2(w[5], w[7], w[4]);
            r4 = interp1(w[5], w[6]);
        }
        96 | 97 | 100 | 101 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            r3 = interp1(w[5], w[4]);
            r4 = interp2(w[5], w[9], w[6]);
        }
        40 | 44 | 168 | 172 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            r3 = interp1(w[5], w[8]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        9 | 13 | 137 | 141 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        18 | 50 => {
            r1 = interp2(w[5], w[1], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = interp1(w[5], w[3]);
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        80 | 81 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            r3 = interp2(w[5], w[7], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = interp1(w[5], w[9]);
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        72 | 76 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = interp1(w[5], w[7]);
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            r4 = interp2(w[5], w[9], w[6]);
        }
        10 | 138 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = interp1(w[5], w[1]);
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            r2 = interp2(w[5], w[3], w[6]);
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        66 => {
            r1 = interp2(w[5], w[1], w[4]);
            r2 = interp2(w[5], w[3], w[6]);
            r3 = interp2(w[5], w[7], w[4]);
            r4 = interp2(w[5], w[9], w[6]);
        }
        24 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        7 | 39 | 135 => {
            r1 = interp1(w[5], w[4]);
            r2 = interp1(w[5], w[6]);
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        148 | 149 | 180 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp1(w[5], w[2]);
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp1(w[5], w[8]);
        }
        224 | 228 | 225 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            r3 = interp1(w[5], w[4]);
            r4 = interp1(w[5], w[6]);
        }
        41 | 169 | 45 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            r3 = interp1(w[5], w[8]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        22 | 54 => {
            r1 = interp2(w[5], w[1], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        208 | 209 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            r3 = interp2(w[5], w[7], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        104 | 108 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            r4 = interp2(w[5], w[9], w[6]);
        }
        11 | 139 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            r2 = interp2(w[5], w[3], w[6]);
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        19 | 51 => {
            if w[2].into_yuv() != w[6].into_yuv() {
                r1 = interp1(w[5], w[4]);
                r2 = interp1(w[5], w[3]);
            } else {
                r1 = interp6(w[5], w[2], w[4]);
                r2 = interp9(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        146 | 178 => {
            r1 = interp2(w[5], w[1], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = interp1(w[5], w[3]);
                r4 = interp1(w[5], w[8]);
            } else {
                r2 = interp9(w[5], w[2], w[6]);
                r4 = interp6(w[5], w[6], w[8]);
            }
            r3 = interp2(w[5], w[8], w[4]);
        }
        84 | 85 => {
            r1 = interp2(w[5], w[4], w[2]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r2 = interp1(w[5], w[2]);
                r4 = interp1(w[5], w[9]);
            } else {
                r2 = interp6(w[5], w[6], w[2]);
                r4 = interp9(w[5], w[6], w[8]);
            }
            r3 = interp2(w[5], w[7], w[4]);
        }
        112 | 113 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r3 = interp1(w[5], w[4]);
                r4 = interp1(w[5], w[9]);
            } else {
                r3 = interp6(w[5], w[8], w[4]);
                r4 = interp9(w[5], w[6], w[8]);
            }
        }
        200 | 204 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = interp1(w[5], w[7]);
                r4 = interp1(w[5], w[6]);
            } else {
                r3 = interp9(w[5], w[8], w[4]);
                r4 = interp6(w[5], w[8], w[6]);
            }
        }
        73 | 77 => {
            if w[8].into_yuv() != w[4].into_yuv() {
                r1 = interp1(w[5], w[2]);
                r3 = interp1(w[5], w[7]);
            } else {
                r1 = interp6(w[5], w[4], w[2]);
                r3 = interp9(w[5], w[8], w[4]);
            }
            r2 = interp2(w[5], w[2], w[6]);
            r4 = interp2(w[5], w[9], w[6]);
        }
        42 | 170 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = interp1(w[5], w[1]);
                r3 = interp1(w[5], w[8]);
            } else {
                r1 = interp9(w[5], w[4], w[2]);
                r3 = interp6(w[5], w[4], w[8]);
            }
            r2 = interp2(w[5], w[3], w[6]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        14 | 142 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = interp1(w[5], w[1]);
                r2 = interp1(w[5], w[6]);
            } else {
                r1 = interp9(w[5], w[4], w[2]);
                r2 = interp6(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        67 => {
            r1 = interp1(w[5], w[4]);
            r2 = interp2(w[5], w[3], w[6]);
            r3 = interp2(w[5], w[7], w[4]);
            r4 = interp2(w[5], w[9], w[6]);
        }
        70 => {
            r1 = interp2(w[5], w[1], w[4]);
            r2 = interp1(w[5], w[6]);
            r3 = interp2(w[5], w[7], w[4]);
            r4 = interp2(w[5], w[9], w[6]);
        }
        28 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp1(w[5], w[2]);
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        152 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp1(w[5], w[8]);
        }
        194 => {
            r1 = interp2(w[5], w[1], w[4]);
            r2 = interp2(w[5], w[3], w[6]);
            r3 = interp2(w[5], w[7], w[4]);
            r4 = interp1(w[5], w[6]);
        }
        98 => {
            r1 = interp2(w[5], w[1], w[4]);
            r2 = interp2(w[5], w[3], w[6]);
            r3 = interp1(w[5], w[4]);
            r4 = interp2(w[5], w[9], w[6]);
        }
        56 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            r3 = interp1(w[5], w[8]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        25 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        26 | 31 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        82 | 214 => {
            r1 = interp2(w[5], w[1], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[7], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        88 | 248 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        74 | 107 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            r2 = interp2(w[5], w[3], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            r4 = interp2(w[5], w[9], w[6]);
        }
        27 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            r2 = interp1(w[5], w[3]);
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        86 => {
            r1 = interp2(w[5], w[1], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[7], w[4]);
            r4 = interp1(w[5], w[9]);
        }
        216 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            r3 = interp1(w[5], w[7]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        106 => {
            r1 = interp1(w[5], w[1]);
            r2 = interp2(w[5], w[3], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            r4 = interp2(w[5], w[9], w[6]);
        }
        30 => {
            r1 = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        210 => {
            r1 = interp2(w[5], w[1], w[4]);
            r2 = interp1(w[5], w[3]);
            r3 = interp2(w[5], w[7], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        120 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            r4 = interp1(w[5], w[9]);
        }
        75 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            r2 = interp2(w[5], w[3], w[6]);
            r3 = interp1(w[5], w[7]);
            r4 = interp2(w[5], w[9], w[6]);
        }
        29 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp1(w[5], w[2]);
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        198 => {
            r1 = interp2(w[5], w[1], w[4]);
            r2 = interp1(w[5], w[6]);
            r3 = interp2(w[5], w[7], w[4]);
            r4 = interp1(w[5], w[6]);
        }
        184 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            r3 = interp1(w[5], w[8]);
            r4 = interp1(w[5], w[8]);
        }
        99 => {
            r1 = interp1(w[5], w[4]);
            r2 = interp2(w[5], w[3], w[6]);
            r3 = interp1(w[5], w[4]);
            r4 = interp2(w[5], w[9], w[6]);
        }
        57 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            r3 = interp1(w[5], w[8]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        71 => {
            r1 = interp1(w[5], w[4]);
            r2 = interp1(w[5], w[6]);
            r3 = interp2(w[5], w[7], w[4]);
            r4 = interp2(w[5], w[9], w[6]);
        }
        156 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp1(w[5], w[2]);
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp1(w[5], w[8]);
        }
        226 => {
            r1 = interp2(w[5], w[1], w[4]);
            r2 = interp2(w[5], w[3], w[6]);
            r3 = interp1(w[5], w[4]);
            r4 = interp1(w[5], w[6]);
        }
        60 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp1(w[5], w[2]);
            r3 = interp1(w[5], w[8]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        195 => {
            r1 = interp1(w[5], w[4]);
            r2 = interp2(w[5], w[3], w[6]);
            r3 = interp2(w[5], w[7], w[4]);
            r4 = interp1(w[5], w[6]);
        }
        102 => {
            r1 = interp2(w[5], w[1], w[4]);
            r2 = interp1(w[5], w[6]);
            r3 = interp1(w[5], w[4]);
            r4 = interp2(w[5], w[9], w[6]);
        }
        153 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp1(w[5], w[8]);
        }
        58 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = interp1(w[5], w[1]);
            } else {
                r1 = interp7(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = interp1(w[5], w[3]);
            } else {
                r2 = interp7(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[8]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        83 => {
            r1 = interp1(w[5], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = interp1(w[5], w[3]);
            } else {
                r2 = interp7(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[7], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = interp1(w[5], w[9]);
            } else {
                r4 = interp7(w[5], w[6], w[8]);
            }
        }
        92 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp1(w[5], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = interp1(w[5], w[7]);
            } else {
                r3 = interp7(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = interp1(w[5], w[9]);
            } else {
                r4 = interp7(w[5], w[6], w[8]);
            }
        }
        202 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = interp1(w[5], w[1]);
            } else {
                r1 = interp7(w[5], w[4], w[2]);
            }
            r2 = interp2(w[5], w[3], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = interp1(w[5], w[7]);
            } else {
                r3 = interp7(w[5], w[8], w[4]);
            }
            r4 = interp1(w[5], w[6]);
        }
        78 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = interp1(w[5], w[1]);
            } else {
                r1 = interp7(w[5], w[4], w[2]);
            }
            r2 = interp1(w[5], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = interp1(w[5], w[7]);
            } else {
                r3 = interp7(w[5], w[8], w[4]);
            }
            r4 = interp2(w[5], w[9], w[6]);
        }
        154 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = interp1(w[5], w[1]);
            } else {
                r1 = interp7(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = interp1(w[5], w[3]);
            } else {
                r2 = interp7(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp1(w[5], w[8]);
        }
        114 => {
            r1 = interp2(w[5], w[1], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = interp1(w[5], w[3]);
            } else {
                r2 = interp7(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = interp1(w[5], w[9]);
            } else {
                r4 = interp7(w[5], w[6], w[8]);
            }
        }
        89 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = interp1(w[5], w[7]);
            } else {
                r3 = interp7(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = interp1(w[5], w[9]);
            } else {
                r4 = interp7(w[5], w[6], w[8]);
            }
        }
        90 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = interp1(w[5], w[1]);
            } else {
                r1 = interp7(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = interp1(w[5], w[3]);
            } else {
                r2 = interp7(w[5], w[2], w[6]);
            }
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = interp1(w[5], w[7]);
            } else {
                r3 = interp7(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = interp1(w[5], w[9]);
            } else {
                r4 = interp7(w[5], w[6], w[8]);
            }
        }
        55 | 23 => {
            if w[2].into_yuv() != w[6].into_yuv() {
                r1 = interp1(w[5], w[4]);
                r2 = w[5];
            } else {
                r1 = interp6(w[5], w[2], w[4]);
                r2 = interp9(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        182 | 150 => {
            r1 = interp2(w[5], w[1], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
                r4 = interp1(w[5], w[8]);
            } else {
                r2 = interp9(w[5], w[2], w[6]);
                r4 = interp6(w[5], w[6], w[8]);
            }
            r3 = interp2(w[5], w[8], w[4]);
        }
        213 | 212 => {
            r1 = interp2(w[5], w[4], w[2]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r2 = interp1(w[5], w[2]);
                r4 = w[5];
            } else {
                r2 = interp6(w[5], w[6], w[2]);
                r4 = interp9(w[5], w[6], w[8]);
            }
            r3 = interp2(w[5], w[7], w[4]);
        }
        241 | 240 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r3 = interp1(w[5], w[4]);
                r4 = w[5];
            } else {
                r3 = interp6(w[5], w[8], w[4]);
                r4 = interp9(w[5], w[6], w[8]);
            }
        }
        236 | 232 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
                r4 = interp1(w[5], w[6]);
            } else {
                r3 = interp9(w[5], w[8], w[4]);
                r4 = interp6(w[5], w[8], w[6]);
            }
        }
        109 | 105 => {
            if w[8].into_yuv() != w[4].into_yuv() {
                r1 = interp1(w[5], w[2]);
                r3 = w[5];
            } else {
                r1 = interp6(w[5], w[4], w[2]);
                r3 = interp9(w[5], w[8], w[4]);
            }
            r2 = interp2(w[5], w[2], w[6]);
            r4 = interp2(w[5], w[9], w[6]);
        }
        171 | 43 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
                r3 = interp1(w[5], w[8]);
            } else {
                r1 = interp9(w[5], w[4], w[2]);
                r3 = interp6(w[5], w[4], w[8]);
            }
            r2 = interp2(w[5], w[3], w[6]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        143 | 15 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
                r2 = interp1(w[5], w[6]);
            } else {
                r1 = interp9(w[5], w[4], w[2]);
                r2 = interp6(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        124 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp1(w[5], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            r4 = interp1(w[5], w[9]);
        }
        203 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            r2 = interp2(w[5], w[3], w[6]);
            r3 = interp1(w[5], w[7]);
            r4 = interp1(w[5], w[6]);
        }
        62 => {
            r1 = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[8]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        211 => {
            r1 = interp1(w[5], w[4]);
            r2 = interp1(w[5], w[3]);
            r3 = interp2(w[5], w[7], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        118 => {
            r1 = interp2(w[5], w[1], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[4]);
            r4 = interp1(w[5], w[9]);
        }
        217 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            r3 = interp1(w[5], w[7]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        110 => {
            r1 = interp1(w[5], w[1]);
            r2 = interp1(w[5], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            r4 = interp2(w[5], w[9], w[6]);
        }
        155 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            r2 = interp1(w[5], w[3]);
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp1(w[5], w[8]);
        }
        188 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp1(w[5], w[2]);
            r3 = interp1(w[5], w[8]);
            r4 = interp1(w[5], w[8]);
        }
        185 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            r3 = interp1(w[5], w[8]);
            r4 = interp1(w[5], w[8]);
        }
        61 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp1(w[5], w[2]);
            r3 = interp1(w[5], w[8]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        157 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp1(w[5], w[2]);
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp1(w[5], w[8]);
        }
        103 => {
            r1 = interp1(w[5], w[4]);
            r2 = interp1(w[5], w[6]);
            r3 = interp1(w[5], w[4]);
            r4 = interp2(w[5], w[9], w[6]);
        }
        227 => {
            r1 = interp1(w[5], w[4]);
            r2 = interp2(w[5], w[3], w[6]);
            r3 = interp1(w[5], w[4]);
            r4 = interp1(w[5], w[6]);
        }
        230 => {
            r1 = interp2(w[5], w[1], w[4]);
            r2 = interp1(w[5], w[6]);
            r3 = interp1(w[5], w[4]);
            r4 = interp1(w[5], w[6]);
        }
        199 => {
            r1 = interp1(w[5], w[4]);
            r2 = interp1(w[5], w[6]);
            r3 = interp2(w[5], w[7], w[4]);
            r4 = interp1(w[5], w[6]);
        }
        220 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp1(w[5], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = interp1(w[5], w[7]);
            } else {
                r3 = interp7(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        158 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = interp1(w[5], w[1]);
            } else {
                r1 = interp7(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp1(w[5], w[8]);
        }
        234 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = interp1(w[5], w[1]);
            } else {
                r1 = interp7(w[5], w[4], w[2]);
            }
            r2 = interp2(w[5], w[3], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            r4 = interp1(w[5], w[6]);
        }
        242 => {
            r1 = interp2(w[5], w[1], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = interp1(w[5], w[3]);
            } else {
                r2 = interp7(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        59 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = interp1(w[5], w[3]);
            } else {
                r2 = interp7(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[8]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        121 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = interp1(w[5], w[9]);
            } else {
                r4 = interp7(w[5], w[6], w[8]);
            }
        }
        87 => {
            r1 = interp1(w[5], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[7], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = interp1(w[5], w[9]);
            } else {
                r4 = interp7(w[5], w[6], w[8]);
            }
        }
        79 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            r2 = interp1(w[5], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = interp1(w[5], w[7]);
            } else {
                r3 = interp7(w[5], w[8], w[4]);
            }
            r4 = interp2(w[5], w[9], w[6]);
        }
        122 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = interp1(w[5], w[1]);
            } else {
                r1 = interp7(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = interp1(w[5], w[3]);
            } else {
                r2 = interp7(w[5], w[2], w[6]);
            }
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = interp1(w[5], w[9]);
            } else {
                r4 = interp7(w[5], w[6], w[8]);
            }
        }
        94 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = interp1(w[5], w[1]);
            } else {
                r1 = interp7(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = interp1(w[5], w[7]);
            } else {
                r3 = interp7(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = interp1(w[5], w[9]);
            } else {
                r4 = interp7(w[5], w[6], w[8]);
            }
        }
        218 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = interp1(w[5], w[1]);
            } else {
                r1 = interp7(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = interp1(w[5], w[3]);
            } else {
                r2 = interp7(w[5], w[2], w[6]);
            }
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = interp1(w[5], w[7]);
            } else {
                r3 = interp7(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        91 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = interp1(w[5], w[3]);
            } else {
                r2 = interp7(w[5], w[2], w[6]);
            }
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = interp1(w[5], w[7]);
            } else {
                r3 = interp7(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = interp1(w[5], w[9]);
            } else {
                r4 = interp7(w[5], w[6], w[8]);
            }
        }
        229 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            r3 = interp1(w[5], w[4]);
            r4 = interp1(w[5], w[6]);
        }
        167 => {
            r1 = interp1(w[5], w[4]);
            r2 = interp1(w[5], w[6]);
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        173 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            r3 = interp1(w[5], w[8]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        181 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp1(w[5], w[2]);
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp1(w[5], w[8]);
        }
        186 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = interp1(w[5], w[1]);
            } else {
                r1 = interp7(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = interp1(w[5], w[3]);
            } else {
                r2 = interp7(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[8]);
            r4 = interp1(w[5], w[8]);
        }
        115 => {
            r1 = interp1(w[5], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = interp1(w[5], w[3]);
            } else {
                r2 = interp7(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = interp1(w[5], w[9]);
            } else {
                r4 = interp7(w[5], w[6], w[8]);
            }
        }
        93 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp1(w[5], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = interp1(w[5], w[7]);
            } else {
                r3 = interp7(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = interp1(w[5], w[9]);
            } else {
                r4 = interp7(w[5], w[6], w[8]);
            }
        }
        206 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = interp1(w[5], w[1]);
            } else {
                r1 = interp7(w[5], w[4], w[2]);
            }
            r2 = interp1(w[5], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = interp1(w[5], w[7]);
            } else {
                r3 = interp7(w[5], w[8], w[4]);
            }
            r4 = interp1(w[5], w[6]);
        }
        205 | 201 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = interp1(w[5], w[7]);
            } else {
                r3 = interp7(w[5], w[8], w[4]);
            }
            r4 = interp1(w[5], w[6]);
        }
        174 | 46 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = interp1(w[5], w[1]);
            } else {
                r1 = interp7(w[5], w[4], w[2]);
            }
            r2 = interp1(w[5], w[6]);
            r3 = interp1(w[5], w[8]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        179 | 147 => {
            r1 = interp1(w[5], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = interp1(w[5], w[3]);
            } else {
                r2 = interp7(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp1(w[5], w[8]);
        }
        117 | 116 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp1(w[5], w[2]);
            r3 = interp1(w[5], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = interp1(w[5], w[9]);
            } else {
                r4 = interp7(w[5], w[6], w[8]);
            }
        }
        189 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp1(w[5], w[2]);
            r3 = interp1(w[5], w[8]);
            r4 = interp1(w[5], w[8]);
        }
        231 => {
            r1 = interp1(w[5], w[4]);
            r2 = interp1(w[5], w[6]);
            r3 = interp1(w[5], w[4]);
            r4 = interp1(w[5], w[6]);
        }
        126 => {
            r1 = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            r4 = interp1(w[5], w[9]);
        }
        219 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            r2 = interp1(w[5], w[3]);
            r3 = interp1(w[5], w[7]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        125 => {
            if w[8].into_yuv() != w[4].into_yuv() {
                r1 = interp1(w[5], w[2]);
                r3 = w[5];
            } else {
                r1 = interp6(w[5], w[4], w[2]);
                r3 = interp9(w[5], w[8], w[4]);
            }
            r2 = interp1(w[5], w[2]);
            r4 = interp1(w[5], w[9]);
        }
        221 => {
            r1 = interp1(w[5], w[2]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r2 = interp1(w[5], w[2]);
                r4 = w[5];
            } else {
                r2 = interp6(w[5], w[6], w[2]);
                r4 = interp9(w[5], w[6], w[8]);
            }
            r3 = interp1(w[5], w[7]);
        }
        207 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
                r2 = interp1(w[5], w[6]);
            } else {
                r1 = interp9(w[5], w[4], w[2]);
                r2 = interp6(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[7]);
            r4 = interp1(w[5], w[6]);
        }
        238 => {
            r1 = interp1(w[5], w[1]);
            r2 = interp1(w[5], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
                r4 = interp1(w[5], w[6]);
            } else {
                r3 = interp9(w[5], w[8], w[4]);
                r4 = interp6(w[5], w[8], w[6]);
            }
        }
        190 => {
            r1 = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
                r4 = interp1(w[5], w[8]);
            } else {
                r2 = interp9(w[5], w[2], w[6]);
                r4 = interp6(w[5], w[6], w[8]);
            }
            r3 = interp1(w[5], w[8]);
        }
        187 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
                r3 = interp1(w[5], w[8]);
            } else {
                r1 = interp9(w[5], w[4], w[2]);
                r3 = interp6(w[5], w[4], w[8]);
            }
            r2 = interp1(w[5], w[3]);
            r4 = interp1(w[5], w[8]);
        }
        243 => {
            r1 = interp1(w[5], w[4]);
            r2 = interp1(w[5], w[3]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r3 = interp1(w[5], w[4]);
                r4 = w[5];
            } else {
                r3 = interp6(w[5], w[8], w[4]);
                r4 = interp9(w[5], w[6], w[8]);
            }
        }
        119 => {
            if w[2].into_yuv() != w[6].into_yuv() {
                r1 = interp1(w[5], w[4]);
                r2 = w[5];
            } else {
                r1 = interp6(w[5], w[2], w[4]);
                r2 = interp9(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[4]);
            r4 = interp1(w[5], w[9]);
        }
        237 | 233 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp2(w[5], w[2], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp10(w[5], w[8], w[4]);
            }
            r4 = interp1(w[5], w[6]);
        }
        175 | 47 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp10(w[5], w[4], w[2]);
            }
            r2 = interp1(w[5], w[6]);
            r3 = interp1(w[5], w[8]);
            r4 = interp2(w[5], w[6], w[8]);
        }
        183 | 151 => {
            r1 = interp1(w[5], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp10(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[8], w[4]);
            r4 = interp1(w[5], w[8]);
        }
        245 | 244 => {
            r1 = interp2(w[5], w[4], w[2]);
            r2 = interp1(w[5], w[2]);
            r3 = interp1(w[5], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp10(w[5], w[6], w[8]);
            }
        }
        250 => {
            r1 = interp1(w[5], w[1]);
            r2 = interp1(w[5], w[3]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        123 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            r2 = interp1(w[5], w[3]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            r4 = interp1(w[5], w[9]);
        }
        95 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[7]);
            r4 = interp1(w[5], w[9]);
        }
        222 => {
            r1 = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[7]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        252 => {
            r1 = interp2(w[5], w[1], w[2]);
            r2 = interp1(w[5], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp10(w[5], w[6], w[8]);
            }
        }
        249 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp2(w[5], w[3], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp10(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        235 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            r2 = interp2(w[5], w[3], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp10(w[5], w[8], w[4]);
            }
            r4 = interp1(w[5], w[6]);
        }
        111 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp10(w[5], w[4], w[2]);
            }
            r2 = interp1(w[5], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            r4 = interp2(w[5], w[9], w[6]);
        }
        63 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp10(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[8]);
            r4 = interp2(w[5], w[9], w[8]);
        }
        159 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp10(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[7], w[8]);
            r4 = interp1(w[5], w[8]);
        }
        215 => {
            r1 = interp1(w[5], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp10(w[5], w[2], w[6]);
            }
            r3 = interp2(w[5], w[7], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        246 => {
            r1 = interp2(w[5], w[1], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp10(w[5], w[6], w[8]);
            }
        }
        254 => {
            r1 = interp1(w[5], w[1]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp10(w[5], w[6], w[8]);
            }
        }
        253 => {
            r1 = interp1(w[5], w[2]);
            r2 = interp1(w[5], w[2]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp10(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp10(w[5], w[6], w[8]);
            }
        }
        251 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            r2 = interp1(w[5], w[3]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp10(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        239 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp10(w[5], w[4], w[2]);
            }
            r2 = interp1(w[5], w[6]);
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp10(w[5], w[8], w[4]);
            }
            r4 = interp1(w[5], w[6]);
        }
        127 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp10(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp2(w[5], w[2], w[6]);
            }
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp2(w[5], w[8], w[4]);
            }
            r4 = interp1(w[5], w[9]);
        }
        191 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp10(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp10(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[8]);
            r4 = interp1(w[5], w[8]);
        }
        223 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp2(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp10(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[7]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp2(w[5], w[6], w[8]);
            }
        }
        247 => {
            r1 = interp1(w[5], w[4]);
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp10(w[5], w[2], w[6]);
            }
            r3 = interp1(w[5], w[4]);
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp10(w[5], w[6], w[8]);
            }
        }
        255 => {
            if w[4].into_yuv() != w[2].into_yuv() {
                r1 = w[5];
            } else {
                r1 = interp10(w[5], w[4], w[2]);
            }
            if w[2].into_yuv() != w[6].into_yuv() {
                r2 = w[5];
            } else {
                r2 = interp10(w[5], w[2], w[6]);
            }
            if w[8].into_yuv() != w[4].into_yuv() {
                r3 = w[5];
            } else {
                r3 = interp10(w[5], w[8], w[4]);
            }
            if w[6].into_yuv() != w[8].into_yuv() {
                r4 = w[5];
            } else {
                r4 = interp10(w[5], w[6], w[8]);
            }
        }
    }

    [r1, r2, r3, r4]
}

#[cfg(test)]
mod tests {
    use test_util::{data::read_nes_smb, snap::ImageSnapshot};

    #[test]
    fn hq2x() {
        let original = read_nes_smb();

        super::hq2x(&original).snapshot("px_up_hq2x");
    }
}
