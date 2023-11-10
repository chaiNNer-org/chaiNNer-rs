#![allow(clippy::comparison_chain)]

use std::ops::{Add, Mul};

use image_core::Image;

use super::util::write_2x;

// Source was adapted from the original C++ code here:
// https://vdnoort.home.xs4all.nl/emulation/2xsai/
//
// 2xSaI is Copyright (c) 1999-2001 by Derek Liauw Kie Fa.
// 2xSaI is free under GPL

pub fn sai_2x<T>(src: &Image<T>) -> Image<T>
where
    T: Default + Copy + PartialEq + Add<T, Output = T> + Mul<f32, Output = T>,
{
    let mut result = Image::from_const(src.size().scale(2.0), T::default());

    let w = src.width();
    let h = src.height();
    let src = src.data();

    let dest = result.data_mut();

    for y in 0..h {
        let y_m1 = y.saturating_sub(1);
        let y_p1 = (y + 1).min(h - 1);
        let y_p2 = (y + 2).min(h - 1);

        for x in 0..w {
            let x_m1 = x.saturating_sub(1);
            let x_p1 = (x + 1).min(w - 1);
            let x_p2 = (x + 2).min(w - 1);

            // I E F J
            // G A B K
            // H C D L
            // M N O P
            let color_i = src[y_m1 * w + x_m1];
            let color_e = src[y_m1 * w + x];
            let color_f = src[y_m1 * w + x_p1];
            let color_j = src[y_m1 * w + x_p2];

            let color_g = src[y * w + x_m1];
            let color_a = src[y * w + x];
            let color_b = src[y * w + x_p1];
            let color_k = src[y * w + x_p2];

            let color_h = src[y_p1 * w + x_m1];
            let color_c = src[y_p1 * w + x];
            let color_d = src[y_p1 * w + x_p1];
            let color_l = src[y_p1 * w + x_p2];

            let color_m = src[y_p2 * w + x_m1];
            let color_n = src[y_p2 * w + x];
            let color_o = src[y_p2 * w + x_p1];
            // let color_p = src[y_p2 * w + x_p2];

            let r1 = color_a;
            let product;
            let product1;
            let product2;

            if color_a == color_d && color_b != color_c {
                if (color_a == color_e && color_b == color_l)
                    || (color_a == color_c
                        && color_a == color_f
                        && color_b != color_e
                        && color_b == color_j)
                {
                    product = color_a;
                } else {
                    product = avg2(color_a, color_b);
                }

                if (color_a == color_g && color_c == color_o)
                    || (color_a == color_b
                        && color_a == color_h
                        && color_g != color_c
                        && color_c == color_m)
                {
                    product1 = color_a;
                } else {
                    product1 = avg2(color_a, color_c);
                }
                product2 = color_a;
            } else if color_b == color_c && color_a != color_d {
                if (color_b == color_f && color_a == color_h)
                    || (color_b == color_e
                        && color_b == color_d
                        && color_a != color_f
                        && color_a == color_i)
                {
                    product = color_b;
                } else {
                    product = avg2(color_a, color_b);
                }

                if (color_c == color_h && color_a == color_f)
                    || (color_c == color_g
                        && color_c == color_d
                        && color_a != color_h
                        && color_a == color_i)
                {
                    product1 = color_c;
                } else {
                    product1 = avg2(color_a, color_c);
                }
                product2 = color_b;
            } else if color_a == color_d && color_b == color_c {
                if color_a == color_b {
                    product = color_a;
                    product1 = color_a;
                    product2 = color_a;
                } else {
                    let mut r = 0;
                    product1 = avg2(color_a, color_c);
                    product = avg2(color_a, color_b);

                    r += get_result_1(color_a, color_b, color_g, color_e);
                    r += get_result_2(color_b, color_a, color_k, color_f);
                    r += get_result_2(color_b, color_a, color_h, color_n);
                    r += get_result_1(color_a, color_b, color_l, color_o);

                    product2 = if r > 0 {
                        color_a
                    } else if r < 0 {
                        color_b
                    } else {
                        avg4(color_a, color_b, color_c, color_d)
                    };
                }
            } else {
                product2 = avg4(color_a, color_b, color_c, color_d);

                if color_a == color_c
                    && color_a == color_f
                    && color_b != color_e
                    && color_b == color_j
                {
                    product = color_a;
                } else if color_b == color_e
                    && color_b == color_d
                    && color_a != color_f
                    && color_a == color_i
                {
                    product = color_b;
                } else {
                    product = avg2(color_a, color_b);
                }

                if color_a == color_b
                    && color_a == color_h
                    && color_g != color_c
                    && color_c == color_m
                {
                    product1 = color_a;
                } else if color_c == color_g
                    && color_c == color_d
                    && color_a != color_h
                    && color_a == color_i
                {
                    product1 = color_c;
                } else {
                    product1 = avg2(color_a, color_c);
                }
            }

            write_2x(dest, w, x, y, [r1, product, product1, product2]);
        }
    }

    result
}

pub fn super_eagle_2x<T>(src: &Image<T>) -> Image<T>
where
    T: Default + Copy + PartialEq + Add<T, Output = T> + Mul<f32, Output = T>,
{
    let mut result = Image::from_const(src.size().scale(2.0), T::default());

    let w = src.width();
    let h = src.height();
    let src = src.data();

    let dest = result.data_mut();

    for y in 0..h {
        let y_m1 = y.saturating_sub(1);
        let y_p1 = (y + 1).min(h - 1);
        let y_p2 = (y + 2).min(h - 1);

        for x in 0..w {
            let x_m1 = x.saturating_sub(1);
            let x_p1 = (x + 1).min(w - 1);
            let x_p2 = (x + 2).min(w - 1);

            let color_b1 = src[y_m1 * w + x];
            let color_b2 = src[y_m1 * w + x_p1];

            let color4 = src[y * w + x_m1];
            let color5 = src[y * w + x];
            let color6 = src[y * w + x_p1];
            let color_s2 = src[y * w + x_p2];

            let color1 = src[y_p1 * w + x_m1];
            let color2 = src[y_p1 * w + x];
            let color3 = src[y_p1 * w + x_p1];
            let color_s1 = src[y_p1 * w + x_p2];

            let color_a1 = src[y_p2 * w + x];
            let color_a2 = src[y_p2 * w + x_p1];

            let product1a;
            let product1b;
            let product2a;
            let product2b;

            if color2 == color6 && color5 != color3 {
                product1b = color2;
                product2a = color2;
                if color1 == color2 || color6 == color_b2 {
                    let i = avg2(color2, color5);
                    product1a = avg2(color2, i);
                } else {
                    product1a = avg2(color5, color6);
                }

                if color6 == color_s2 || color2 == color_a1 {
                    let i = avg2(color2, color3);
                    product2b = avg2(color2, i);
                } else {
                    product2b = avg2(color2, color3);
                }
            } else if color5 == color3 && color2 != color6 {
                product2b = color5;
                product1a = color5;

                if color_b1 == color5 || color3 == color_s1 {
                    let i = avg2(color5, color6);
                    product1b = avg2(color5, i);
                } else {
                    product1b = avg2(color5, color6);
                }

                if color3 == color_a2 || color4 == color5 {
                    let i = avg2(color5, color2);
                    product2a = avg2(color5, i);
                } else {
                    product2a = avg2(color2, color3);
                }
            } else if color5 == color3 && color2 == color6 {
                let mut r = 0;

                r += get_result_1(color6, color5, color1, color_a1);
                r += get_result_1(color6, color5, color4, color_b1);
                r += get_result_1(color6, color5, color_a2, color_s1);
                r += get_result_1(color6, color5, color_b2, color_s2);

                if r > 0 {
                    product1b = color2;
                    product2a = color2;
                    let i = avg2(color5, color6);
                    product1a = i;
                    product2b = i;
                } else if r < 0 {
                    product2b = color5;
                    product1a = color5;
                    let i = avg2(color5, color6);
                    product1b = i;
                    product2a = i;
                } else {
                    product2b = color5;
                    product1a = color5;
                    product1b = color2;
                    product2a = color2;
                }
            } else {
                let i = avg2(color2, color6);
                product2b = avg4(color3, color3, color3, i);
                product1a = avg4(color5, color5, color5, i);

                let i = avg2(color5, color3);
                product2a = avg4(color2, color2, color2, i);
                product1b = avg4(color6, color6, color6, i);
            }

            write_2x(dest, w, x, y, [product1a, product1b, product2a, product2b]);
        }
    }

    result
}

pub fn super_sai_2x<T>(src: &Image<T>) -> Image<T>
where
    T: Default + Copy + PartialEq + Add<T, Output = T> + Mul<f32, Output = T>,
{
    let mut result = Image::from_const(src.size().scale(2.0), T::default());

    let w = src.width();
    let h = src.height();
    let src = src.data();

    let dest = result.data_mut();

    for y in 0..h {
        let y_m1 = y.saturating_sub(1);
        let y_p1 = (y + 1).min(h - 1);
        let y_p2 = (y + 2).min(h - 1);

        for x in 0..w {
            let x_m1 = x.saturating_sub(1);
            let x_p1 = (x + 1).min(w - 1);
            let x_p2 = (x + 2).min(w - 1);

            let color_b0 = src[y_m1 * w + x_m1];
            let color_b1 = src[y_m1 * w + x];
            let color_b2 = src[y_m1 * w + x_p1];
            let color_b3 = src[y_m1 * w + x_p2];

            let color4 = src[y * w + x_m1];
            let color5 = src[y * w + x];
            let color6 = src[y * w + x_p1];
            let color_s2 = src[y * w + x_p2];

            let color1 = src[y_p1 * w + x_m1];
            let color2 = src[y_p1 * w + x];
            let color3 = src[y_p1 * w + x_p1];
            let color_s1 = src[y_p1 * w + x_p2];

            let color_a0 = src[y_p2 * w + x_m1];
            let color_a1 = src[y_p2 * w + x];
            let color_a2 = src[y_p2 * w + x_p1];
            let color_a3 = src[y_p2 * w + x_p2];

            #[allow(clippy::needless_late_init)]
            let product1a;
            let product1b;
            #[allow(clippy::needless_late_init)]
            let product2a;
            let product2b;

            if color2 == color6 && color5 != color3 {
                product2b = color2;
                product1b = color2;
            } else if color5 == color3 && color2 != color6 {
                product2b = color5;
                product1b = color5;
            } else if color5 == color3 && color2 == color6 {
                let mut r = 0;

                r += get_result_1(color6, color5, color1, color_a1);
                r += get_result_1(color6, color5, color4, color_b1);
                r += get_result_1(color6, color5, color_a2, color_s1);
                r += get_result_1(color6, color5, color_b2, color_s2);

                if r > 0 {
                    product2b = color6;
                    product1b = color6;
                } else if r < 0 {
                    product2b = color5;
                    product1b = color5;
                } else {
                    let i = avg2(color5, color6);
                    product2b = i;
                    product1b = i;
                }
            } else {
                if color6 == color3
                    && color3 == color_a1
                    && color2 != color_a2
                    && color3 != color_a0
                {
                    product2b = avg4(color3, color3, color3, color2);
                } else if color5 == color2
                    && color2 == color_a2
                    && color_a1 != color3
                    && color2 != color_a3
                {
                    product2b = avg4(color2, color2, color2, color3);
                } else {
                    product2b = avg2(color2, color3);
                }

                if color6 == color3
                    && color6 == color_b1
                    && color5 != color_b2
                    && color6 != color_b0
                {
                    product1b = avg4(color6, color6, color6, color5);
                } else if color5 == color2
                    && color5 == color_b2
                    && color_b1 != color6
                    && color5 != color_b3
                {
                    product1b = avg4(color6, color5, color5, color5);
                } else {
                    product1b = avg2(color5, color6);
                }
            }

            if color5 == color3 && color2 != color6 && color4 == color5 && color5 != color_a2
                || color5 == color1 && color6 == color5 && color4 != color2 && color5 != color_a0
            {
                product2a = avg2(color2, color5);
            } else {
                product2a = color2;
            }

            if color2 == color6 && color5 != color3 && color1 == color2 && color2 != color_b2
                || color4 == color2 && color3 == color2 && color1 != color5 && color2 != color_b0
            {
                product1a = avg2(color2, color5);
            } else {
                product1a = color5;
            }

            write_2x(dest, w, x, y, [product1a, product1b, product2a, product2b]);
        }
    }

    result
}

fn avg2<T: Add<T, Output = T> + Mul<f32, Output = T>>(a: T, b: T) -> T {
    (a + b) * 0.5
}
fn avg4<T: Add<T, Output = T> + Mul<f32, Output = T>>(a: T, b: T, c: T, d: T) -> T {
    (a + b + c + d) * 0.25
}

fn get_result_1<T: PartialEq>(a: T, b: T, c: T, d: T) -> i32 {
    let mut x = 0;
    let mut y = 0;
    let mut r = 0;
    if a == c {
        x += 1;
    } else if b == c {
        y += 1;
    }
    if a == d {
        x += 1;
    } else if b == d {
        y += 1;
    }
    if x <= 1 {
        r += 1
    }
    if y <= 1 {
        r -= 1
    }
    r
}
fn get_result_2<T: PartialEq>(a: T, b: T, c: T, d: T) -> i32 {
    let mut x = 0;
    let mut y = 0;
    let mut r = 0;
    if a == c {
        x += 1;
    } else if b == c {
        y += 1;
    }
    if a == d {
        x += 1;
    } else if b == d {
        y += 1;
    }
    if x <= 1 {
        r -= 1
    }
    if y <= 1 {
        r += 1
    }
    r
}

#[cfg(test)]
mod tests {
    use test_util::{data::read_nes_smb, snap::ImageSnapshot};

    #[test]
    fn sai() {
        let original = read_nes_smb();

        super::sai_2x(&original).snapshot("px_up_sai_2x");
        super::super_sai_2x(&original).snapshot("px_up_super_sai_2x");
    }

    #[test]
    fn super_eagle() {
        let original = read_nes_smb();

        super::super_eagle_2x(&original).snapshot("px_up_super_eagle_2x");
    }
}
