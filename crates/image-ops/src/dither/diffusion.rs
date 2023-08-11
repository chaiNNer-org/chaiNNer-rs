use image_core::Image;

use crate::util::from_const;

use super::{Diffuser, DiffusionAlgorithm, Pixel, Quantizer};

pub fn error_diffusion_dither<P: Pixel>(
    src: &mut Image<P>,
    algorithm: impl DiffusionAlgorithm,
    quant: &impl Quantizer<P, P>,
) {
    let w = src.width();
    let h = src.height();
    let data = src.data_mut();

    let mut error_rows = ErrorRows::<P>::new(w);

    for y in 0..h {
        error_rows.rotate();

        for x in 0..w {
            let index = y * w + x;
            let error_x = x + ERROR_ROW_OFFSET;

            let color = quant.combine_error(data[index], error_rows.0[error_x]);
            let nearest = quant.get_nearest_color(color);
            let error = quant.get_error(color, nearest);

            data[index] = nearest;

            algorithm.define_weights(StandardDiffuser {
                rows: [&mut *error_rows.0, &mut *error_rows.1, &mut *error_rows.2],
                x: error_x,
                error,
            });
        }
    }
}

pub fn error_diffusion_dither_map<P: Pixel, N>(
    src: &Image<P>,
    algorithm: impl DiffusionAlgorithm,
    quant: &impl Quantizer<P, N>,
    out: Option<Image<N>>,
) -> Image<N>
where
    N: Clone + Default,
{
    let mut dest = from_const(src.size(), Default::default(), out);
    let dest_data = dest.data_mut();

    let w = src.width();
    let h = src.height();
    let data = src.data();

    let mut error_rows = ErrorRows::<P>::new(w);

    for y in 0..h {
        error_rows.rotate();

        for x in 0..w {
            let index = y * w + x;
            let error_x = x + ERROR_ROW_OFFSET;

            let color = quant.combine_error(data[index], error_rows.0[error_x]);
            let nearest = quant.get_nearest_color(color);
            let error = quant.get_error(color, nearest.clone());

            dest_data[index] = nearest;

            algorithm.define_weights(StandardDiffuser {
                rows: [&mut *error_rows.0, &mut *error_rows.1, &mut *error_rows.2],
                x: error_x,
                error,
            });
        }
    }

    dest
}

const ERROR_ROW_OFFSET: usize = 2;

struct ErrorRows<P>(Box<[P]>, Box<[P]>, Box<[P]>);

impl<P: Clone + Default> ErrorRows<P> {
    pub fn new(width: usize) -> Self {
        Self(
            vec![Default::default(); width + ERROR_ROW_OFFSET * 2].into_boxed_slice(),
            vec![Default::default(); width + ERROR_ROW_OFFSET * 2].into_boxed_slice(),
            vec![Default::default(); width + ERROR_ROW_OFFSET * 2].into_boxed_slice(),
        )
    }

    pub fn rotate(&mut self) {
        std::mem::swap(&mut self.0, &mut self.1);
        std::mem::swap(&mut self.1, &mut self.2);
        self.2.fill(Default::default());
    }
}

struct StandardDiffuser<'a, P: Pixel> {
    rows: [&'a mut [P]; 3],
    x: usize,
    error: P,
}
impl<'a, P: Pixel> Diffuser for StandardDiffuser<'a, P> {
    #[inline(always)]
    fn assign_weight(&mut self, y: usize, x: isize, weight: f32) {
        assert!(y < 3);
        assert!(-(ERROR_ROW_OFFSET as isize) <= x && x <= ERROR_ROW_OFFSET as isize);

        let x = (self.x as isize + x) as usize;
        self.rows[y][x] += self.error * weight;
    }
}

#[cfg(test)]
mod tests {
    use super::{super::*, *};
    use test_util::{
        data::{read_flower, read_flower_palette},
        snap::ImageSnapshot,
    };

    #[test]
    fn error_diffusion() {
        let mut original = read_flower();
        error_diffusion_dither(&mut original, FloydSteinberg, &ChannelQuantization::new(4));
        original.snapshot("error_diffusion_fs_4");
    }
    #[test]
    fn error_diffusion_map() {
        let original = read_flower();

        error_diffusion_dither_map(
            &original,
            FloydSteinberg,
            &ChannelQuantization::new(2),
            None,
        )
        .snapshot("error_diffusion_map_fs_2");

        error_diffusion_dither_map(
            &original,
            FloydSteinberg,
            &ChannelQuantization::new(4),
            None,
        )
        .snapshot("error_diffusion_map_fs_4");

        error_diffusion_dither_map(
            &original,
            JarvisJudiceNinke,
            &ChannelQuantization::new(4),
            None,
        )
        .snapshot("error_diffusion_map_jjn_4");

        error_diffusion_dither_map(
            &original,
            FloydSteinberg,
            &ChannelQuantization::new(16),
            None,
        )
        .snapshot("error_diffusion_map_flower_fs_16");
        error_diffusion_dither_map(&original, Atkinson, &ChannelQuantization::new(16), None)
            .snapshot("error_diffusion_map_atk_16");
    }

    #[test]
    fn error_diffusion_color_palette() {
        let img = read_flower();
        let palette_img = read_flower_palette();

        let palette = ColorPalette::new(RGB, palette_img.row(0).iter().copied(), BoundError);

        error_diffusion_dither_map(&img, FloydSteinberg, &palette, None)
            .snapshot("error_diffusion_palette_fs");
    }
}
