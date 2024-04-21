use image_core::{NDimImage, NDimView};

use crate::util::BiLinear;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AntiAliasing {
    pub extra_smoothness: f32,
}

pub fn binary_threshold(img: &mut NDimImage, threshold: f32, anti_aliasing: Option<AntiAliasing>) {
    if let Some(AntiAliasing {
        extra_smoothness: extra_smooth,
    }) = anti_aliasing
    {
        // if anti-aliasing is enabled, we need to do some extra work
        let original = img.clone();

        for p in img.data_mut() {
            *p = if *p > threshold { 1.0 } else { 0.0 };
        }

        let c = img.channels();
        for i in 0..c {
            binary_threshold_aa(original.view(), img, threshold, i, c, extra_smooth);
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
    extra_smoothness: f32,
) {
    let w = img.width();
    let h = img.height();

    let img_data = img.data();
    let data = dest.data_mut();

    // find edges
    let edges = find_edges(data, w, h, offset, stride);

    // do the anti-aliasing
    let corner_size = (0.5 + extra_smoothness / 2.0).clamp(0.5, 1.0);
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
            .get_top_corner(corner_size);
            let q_tr = BiLinear {
                x0y0: p_cc,
                x1y0: p_cr,
                x0y1: p_tc,
                x1y1: p_tr,
            }
            .get_top_corner(corner_size);
            let q_bl = BiLinear {
                x0y0: p_cc,
                x1y0: p_cl,
                x0y1: p_bc,
                x1y1: p_bl,
            }
            .get_top_corner(corner_size);
            let q_br = BiLinear {
                x0y0: p_cc,
                x1y0: p_cr,
                x0y1: p_bc,
                x1y1: p_br,
            }
            .get_top_corner(corner_size);

            let sum_area = q_tl.get_area(threshold)
                + q_tr.get_area(threshold)
                + q_bl.get_area(threshold)
                + q_br.get_area(threshold);
            data[(y * w + x) * stride + offset] = sum_area * 0.25;
        }
    }
}
fn find_edges(data: &[f32], w: usize, h: usize, offset: usize, stride: usize) -> Box<[bool]> {
    let mut edges = vec![false; w * h].into_boxed_slice();

    for y in 0..h {
        let row_offset = y * w;
        let row_offset_m1 = y.saturating_sub(1) * w;
        let row_offset_p1 = (y + 1).min(h - 1) * w;

        for x in 1..w {
            let i1 = row_offset + x;
            let i0 = i1 - 1;

            let p = data[i0 * stride + offset];
            let n = data[i1 * stride + offset];
            if p != n {
                edges[i0] = true;
                edges[i1] = true;
                edges[row_offset_m1 + x - 1] = true;
                edges[row_offset_m1 + x] = true;
                edges[row_offset_p1 + x - 1] = true;
                edges[row_offset_p1 + x] = true;
            }
        }
    }
    for x in 0..w {
        let col = x;
        let col_m1 = x.saturating_sub(1);
        let col_p1 = (x + 1).min(w - 1);

        for y in 1..h {
            let row0 = (y - 1) * w;
            let row1 = y * w;
            let i0 = row0 + col;
            let i1 = row1 + col;

            let p = data[i0 * stride + offset];
            let n = data[i1 * stride + offset];
            if p != n {
                edges[i0] = true;
                edges[i1] = true;
                edges[row0 + col_m1] = true;
                edges[row1 + col_m1] = true;
                edges[row0 + col_p1] = true;
                edges[row1 + col_p1] = true;
            }
        }
    }

    edges
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
        super::binary_threshold(&mut original, 0.5, None);
        original.snapshot("threshold_flower");

        let mut original: NDimImage = read_at_sdf().into();
        super::binary_threshold(&mut original, 0.5, None);
        original.snapshot("threshold_at_sdf");
    }

    #[test]
    fn binary_threshold_aa() {
        let mut original: NDimImage = read_flower().into();
        super::binary_threshold(
            &mut original,
            0.5,
            Some(super::AntiAliasing {
                extra_smoothness: 0.0,
            }),
        );
        original.snapshot("threshold_aa_flower");

        let mut original: NDimImage = read_at_sdf().into();
        super::binary_threshold(
            &mut original,
            0.5,
            Some(super::AntiAliasing {
                extra_smoothness: 0.0,
            }),
        );
        original.snapshot("threshold_aa_0_at_sdf");

        let mut original: NDimImage = read_at_sdf().into();
        super::binary_threshold(
            &mut original,
            0.5,
            Some(super::AntiAliasing {
                extra_smoothness: 0.2,
            }),
        );
        original.snapshot("threshold_aa_20_at_sdf");

        let mut original: NDimImage = read_at_sdf().into();
        super::binary_threshold(
            &mut original,
            0.5,
            Some(super::AntiAliasing {
                extra_smoothness: 0.4,
            }),
        );
        original.snapshot("threshold_aa_40_at_sdf");

        let mut original: NDimImage = read_at_sdf().into();
        super::binary_threshold(
            &mut original,
            0.5,
            Some(super::AntiAliasing {
                extra_smoothness: 0.6,
            }),
        );
        original.snapshot("threshold_aa_60_at_sdf");

        let mut original: NDimImage = read_at_sdf().into();
        super::binary_threshold(
            &mut original,
            0.5,
            Some(super::AntiAliasing {
                extra_smoothness: 0.8,
            }),
        );
        original.snapshot("threshold_aa_80_at_sdf");

        let mut original: NDimImage = read_at_sdf().into();
        super::binary_threshold(
            &mut original,
            0.5,
            Some(super::AntiAliasing {
                extra_smoothness: 1.0,
            }),
        );
        original.snapshot("threshold_aa_100_at_sdf");
    }
}
