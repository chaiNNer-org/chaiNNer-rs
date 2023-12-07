use image_core::{NDimImage, NDimView};

use crate::util::{move_range, BiLinear};

pub fn binary_threshold(img: &mut NDimImage, threshold: f32, anti_aliasing: bool) {
    if anti_aliasing {
        // if anti-aliasing is enabled, we need to do some extra work
        let original = img.clone();

        for p in img.data_mut() {
            *p = if *p > threshold { 1.0 } else { 0.0 };
        }

        let c = img.channels();
        for i in 0..c {
            binary_threshold_aa(original.view(), img, threshold, i, c);
        }
    } else {
        for p in img.data_mut() {
            *p = if *p > threshold { 1.0 } else { 0.0 };
        }
    }
}

fn binary_threshold_aa(
    img: NDimView,
    dest: &mut NDimImage,
    threshold: f32,
    offset: usize,
    stride: usize,
) {
    let w = img.width();
    let h = img.height();

    let img_data = img.data();
    let data = dest.data_mut();

    // find edges
    let mut edges = vec![false; w * h].into_boxed_slice();
    for y in 0..h {
        for i in move_range(&(1..w), y * w) {
            let i0 = i - 1;
            let i1 = i;

            let p = data[i0 * stride + offset];
            let n = data[i1 * stride + offset];
            if p != n {
                edges[i0] = true;
                edges[i1] = true;
            }
        }
    }
    for x in 0..w {
        for y in 1..h {
            let i0 = (y - 1) * w + x;
            let i1 = y * w + x;

            let p = data[i0 * stride + offset];
            let n = data[i1 * stride + offset];
            if p != n {
                edges[i0] = true;
                edges[i1] = true;
            }
        }
    }

    // do the anti-aliasing
    for y in 0..h {
        let y_t = y.saturating_sub(1);
        let y_c = y;
        let y_b = usize::min(y + 1, h - 1);
        for x in 0..w {
            if !edges[y * w + x] {
                continue;
            }

            let x_l = x.saturating_sub(1);
            let x_c = x;
            let x_r = usize::min(x + 1, w - 1);

            // the 9 pixels we need to sample
            let p_tl = img_data[(y_t * w + x_l) * stride + offset];
            let p_tc = img_data[(y_t * w + x_c) * stride + offset];
            let p_tr = img_data[(y_t * w + x_r) * stride + offset];
            let p_cl = img_data[(y_c * w + x_l) * stride + offset];
            let p_cc = img_data[(y_c * w + x_c) * stride + offset];
            let p_cr = img_data[(y_c * w + x_r) * stride + offset];
            let p_bl = img_data[(y_b * w + x_l) * stride + offset];
            let p_bc = img_data[(y_b * w + x_c) * stride + offset];
            let p_br = img_data[(y_b * w + x_r) * stride + offset];

            // setup the 4 quadrants for bilinear interpolation
            // quadrants are setup such that sample(0,0) == p_cc
            let q_tl = BiLinear {
                x0y0: p_cc,
                x1y0: p_cl,
                x0y1: p_tc,
                x1y1: p_tl,
            }
            .get_first_quadrant();
            let q_tr = BiLinear {
                x0y0: p_cc,
                x1y0: p_cr,
                x0y1: p_tc,
                x1y1: p_tr,
            }
            .get_first_quadrant();
            let q_bl = BiLinear {
                x0y0: p_cc,
                x1y0: p_cl,
                x0y1: p_bc,
                x1y1: p_bl,
            }
            .get_first_quadrant();
            let q_br = BiLinear {
                x0y0: p_cc,
                x1y0: p_cr,
                x0y1: p_bc,
                x1y1: p_br,
            }
            .get_first_quadrant();

            let sum_area = q_tl.get_area(threshold)
                + q_tr.get_area(threshold)
                + q_bl.get_area(threshold)
                + q_br.get_area(threshold);
            data[(y * w + x) * stride + offset] = sum_area * 0.25;
        }
    }
}

#[cfg(test)]
mod tests {
    use image_core::NDimImage;
    use test_util::{
        data::{read_at_sdf, read_flower},
        snap::ImageSnapshot,
    };

    #[test]
    fn binary_threshold() {
        let mut original: NDimImage = read_flower().into();
        super::binary_threshold(&mut original, 0.5, false);
        original.snapshot("threshold_flower");

        let mut original: NDimImage = read_at_sdf().into();
        super::binary_threshold(&mut original, 0.5, false);
        original.snapshot("threshold_at_sdf");
    }

    #[test]
    fn binary_threshold_aa() {
        let mut original: NDimImage = read_flower().into();
        super::binary_threshold(&mut original, 0.5, true);
        original.snapshot("threshold_aa_flower");

        let mut original: NDimImage = read_at_sdf().into();
        super::binary_threshold(&mut original, 0.5, true);
        original.snapshot("threshold_aa_at_sdf");
    }
}
