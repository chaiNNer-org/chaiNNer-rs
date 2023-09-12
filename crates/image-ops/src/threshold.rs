use image_core::{NDimImage, NDimView};

use crate::util::move_range;

pub fn binary_threshold(img: NDimView, anti_aliasing: bool) -> NDimImage {
    // start with a normal threshold
    let mut dest = NDimImage::new(
        img.shape(),
        img.data()
            .iter()
            .map(|&p| if p >= 0.5 { 1.0 } else { 0.0 })
            .collect(),
    );

    // if anti-aliasing is enabled, we need to do some extra work
    if anti_aliasing {
        match img.channels() {
            1 => binary_threshold_aa(img, &mut dest, 0, 1),
            2 => {
                binary_threshold_aa(img, &mut dest, 0, 2);
                binary_threshold_aa(img, &mut dest, 1, 2);
            }
            3 => {
                binary_threshold_aa(img, &mut dest, 0, 3);
                binary_threshold_aa(img, &mut dest, 1, 3);
                binary_threshold_aa(img, &mut dest, 2, 3);
            }
            4 => {
                binary_threshold_aa(img, &mut dest, 0, 4);
                binary_threshold_aa(img, &mut dest, 1, 4);
                binary_threshold_aa(img, &mut dest, 2, 4);
                binary_threshold_aa(img, &mut dest, 3, 4);
            }
            _ => todo!("Implement anti-aliasing for images with more than 4 channels"),
        }
    }

    dest
}

fn binary_threshold_aa(img: NDimView, dest: &mut NDimImage, offset: usize, stride: usize) {
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

            let sum_area =
                q_tl.get_area(0.5) + q_tr.get_area(0.5) + q_bl.get_area(0.5) + q_br.get_area(0.5);
            data[(y * w + x) * stride + offset] = sum_area * 0.25;
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct BiLinear {
    x0y0: f32,
    x1y0: f32,
    x0y1: f32,
    x1y1: f32,
}

impl BiLinear {
    #[inline]
    fn sample(self, x: f32, y: f32) -> f32 {
        let x0 = self.x0y0 + (self.x1y0 - self.x0y0) * x;
        let x1 = self.x0y1 + (self.x1y1 - self.x0y1) * x;
        x0 + (x1 - x0) * y
    }

    /// Returns the image from x in 0..=0.5 and y in 0..=0.5
    fn get_first_quadrant(self) -> BiLinear {
        BiLinear {
            x0y0: self.sample(0.0, 0.0),
            x1y0: self.sample(0.5, 0.0),
            x0y1: self.sample(0.0, 0.5),
            x1y1: self.sample(0.5, 0.5),
        }
    }
    /// Mirrors the image along the (0,0) -> (1,1) diagonal.
    #[inline]
    fn mirror(self) -> BiLinear {
        BiLinear {
            x0y0: self.x0y0,
            x1y0: self.x0y1,
            x0y1: self.x1y0,
            x1y1: self.x1y1,
        }
    }

    /// Returns the area of the image that is above the given threshold.
    fn get_area(mut self, threshold: f32) -> f32 {
        // since we can mirror the image, we can choose the axis over which we want to integrate.
        // we choose an axis by looking at how much our end points differ.
        let diff_x = (self.x0y0 - self.x1y0).abs() + (self.x0y1 - self.x1y1).abs();
        let diff_y = (self.x0y0 - self.x0y1).abs() + (self.x1y0 - self.x1y1).abs();
        if diff_x < diff_y {
            self = self.mirror()
        }

        // we can rewrite the sample function as:
        //   sample(x,y) = x*y*a + x*b + y*c + d
        // for the following values of a,b,c,d:
        let a = self.x1y1 - self.x1y0 - self.x0y1 + self.x0y0;
        let b = self.x1y0 - self.x0y0;
        let c = self.x0y1 - self.x0y0;
        let d = self.x0y0;

        // The idea here is that we will hold y constant, and then analytically solve the area for that thin strip.
        // We then integrate that area over all y values. Or rather, we approximate the integral.

        let area_for_y = |y: f32| {
            // We are searching for x such that x*y*a + x*b + y*c + d = t.
            // Since that x might not exist within the range 0..=1, we're gonna evaluate the points for x=0 and x=1, and then go from there
            let m = y * a + b;
            let x0 = y * c + d - threshold;
            let x1 = m + x0;

            if x0 < 0.0 && x1 < 0.0 {
                // the strip is entirely below the threshold
                return 0.0;
            }
            if x0 >= 0.0 && x1 >= 0.0 {
                // the strip is entirely above the threshold
                return 1.0;
            }

            // the strip is partially above the threshold
            let x = -x0 / m;
            if x0 >= 0.0 {
                x
            } else {
                1.0 - x
            }
        };

        // integrate over y
        let mut area = 0.0;
        const SAMPLES: usize = 6;
        const SAMPLE_AREA: f32 = 1.0 / SAMPLES as f32;

        let mut last_strip = area_for_y(0.0);
        for i in 1..=SAMPLES {
            let y = i as f32 * SAMPLE_AREA;
            let strip = area_for_y(y);
            area += (strip + last_strip) * 0.5 * SAMPLE_AREA;
            last_strip = strip;
        }

        area
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
        let original: NDimImage = read_flower().into();
        let result = super::binary_threshold(original.view(), false);
        result.snapshot("threshold_flower");

        let original: NDimImage = read_at_sdf().into();
        let result = super::binary_threshold(original.view(), false);
        result.snapshot("threshold_at_sdf");
    }

    #[test]
    fn binary_threshold_aa() {
        let original: NDimImage = read_flower().into();
        let result = super::binary_threshold(original.view(), true);
        result.snapshot("threshold_aa_flower");

        let original: NDimImage = read_at_sdf().into();
        let result = super::binary_threshold(original.view(), true);
        result.snapshot("threshold_aa_at_sdf");
    }

    #[test]
    fn bilinear_area() {
        #[allow(unused)]
        fn get_true_area(b: super::BiLinear) -> f32 {
            const SAMPLES: u64 = 1000;
            let offset = 1.0 / SAMPLES as f32 / 2.0;
            let mut sum: u64 = 0;
            for y in 0..SAMPLES {
                let y = y as f32 / SAMPLES as f32 + offset;
                for x in 0..=SAMPLES {
                    let x = x as f32 / SAMPLES as f32 + offset;
                    sum += if b.sample(x, y) >= 0.5 { 1 } else { 0 };
                }
            }
            sum as f32 / (SAMPLES * SAMPLES) as f32
        }

        // This one has a funny shape. It looks roughly like this for a threshold of 0.5:
        //   1 1 1 0 0 0 0 0
        //   1 1 1 0 0 0 0 0
        //   1 1 1 1 1 1 1 0
        //   0 0 1 1 1 1 1 1
        //   0 0 1 1 1 1 1 1
        //   0 0 1 1 1 1 1 1
        //   0 0 1 1 1 1 1 1
        //   0 0 1 1 1 1 1 1
        // It's area is around 0.65
        let b1 = super::BiLinear {
            x0y0: 0.63,
            x1y0: 0.27,
            x0y1: 0.28,
            x1y1: 1.0,
        };

        assert!((b1.get_area(0.5) - 0.65).abs() < 0.04);

        assert_eq!(
            super::BiLinear {
                x0y0: 0.0,
                x1y0: 0.0,
                x0y1: 0.0,
                x1y1: 0.0,
            }
            .get_area(0.5),
            0.0
        );
        assert_eq!(
            super::BiLinear {
                x0y0: 1.0,
                x1y0: 1.0,
                x0y1: 1.0,
                x1y1: 1.0,
            }
            .get_area(0.5),
            1.0
        );
        assert_eq!(
            super::BiLinear {
                x0y0: 1.0,
                x1y0: 1.0,
                x0y1: 0.0,
                x1y1: 0.0,
            }
            .get_area(0.5),
            0.5
        );
        assert_eq!(
            super::BiLinear {
                x0y0: 1.0,
                x0y1: 0.0,
                x1y0: 1.0,
                x1y1: 0.0,
            }
            .get_area(0.5),
            0.5
        );
        assert_eq!(
            super::BiLinear {
                x0y0: 1.0,
                x0y1: 0.0,
                x1y1: 0.0,
                x1y0: 1.0,
            }
            .get_area(0.5),
            0.5
        );
        assert_eq!(
            super::BiLinear {
                x0y1: 0.0,
                x0y0: 1.0,
                x1y1: 0.0,
                x1y0: 1.0,
            }
            .get_area(0.5),
            0.5
        );
    }
}
