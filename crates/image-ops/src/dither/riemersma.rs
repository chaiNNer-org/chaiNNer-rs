use image_core::Image;

use crate::util::from_const;

use super::{Pixel, Quantizer};

pub fn riemersma_dither<P: Pixel>(
    src: &mut Image<P>,
    history_length: usize,
    decay_ratio: f32,
    quant: &impl Quantizer<P, P>,
) {
    let w = src.width();
    let h = src.height();
    let data = src.data_mut();

    let base = f32::exp(decay_ratio.ln() / (history_length as f32 - 1.0));
    assert!(0.0 < base && base < 1.0);

    let mut history: Box<[P]> = vec![Default::default(); history_length].into_boxed_slice();
    let history = &mut *history;
    let mut history_index = 0;

    for (x, y) in zhang_hilbert::ArbHilbertScan32::new([w as u32, h as u32])
        .map(|[x, y]| (x as usize, y as usize))
    {
        let mut current_error = P::default();
        for error in history.iter() {
            current_error += *error;
        }
        for error in history.iter_mut() {
            *error = *error * base;
        }

        let index = y * w + x;

        let original = data[index];
        let color = quant.combine_error(original, current_error);
        let nearest = quant.get_nearest_color(color);
        let error = quant.get_error(original, nearest);

        data[index] = nearest;

        history[history_index] = error;
        history_index = (history_index + 1) % history_length;
    }
}

pub fn riemersma_dither_map<P: Pixel, N>(
    src: &Image<P>,
    history_length: usize,
    decay_ratio: f32,
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

    let base = 1.0 / f32::exp((1.0 / decay_ratio).ln() / (history_length as f32 - 1.0));
    assert!(0.0 < base && base < 1.0);

    let mut history: Box<[P]> = vec![Default::default(); history_length].into_boxed_slice();
    let history = &mut *history;
    let mut history_index = 0;

    for (x, y) in zhang_hilbert::ArbHilbertScan32::new([w as u32, h as u32])
        .map(|[x, y]| (x as usize, y as usize))
    {
        let mut current_error = P::default();
        for error in history.iter() {
            current_error += *error;
        }
        for error in history.iter_mut() {
            *error = *error * base;
        }

        let index = y * w + x;

        let original = data[index];
        let color = quant.combine_error(original, current_error);
        let nearest = quant.get_nearest_color(color);
        let error = quant.get_error(original, nearest.clone());

        dest_data[index] = nearest;

        history[history_index] = error;
        history_index = (history_index + 1) % history_length;
    }

    dest
}

#[cfg(test)]
mod tests {
    use super::{super::*, *};
    use test_util::{
        data::{read_flower, read_flower_palette},
        snap::ImageSnapshot,
    };

    #[test]
    fn riemersma() {
        let mut original = read_flower();
        riemersma_dither(&mut original, 16, 1.0 / 16.0, &ChannelQuantization::new(4));
        original.snapshot("riemersma_flower_4");
    }
    #[test]
    fn riemersma_map() {
        let original = read_flower();

        riemersma_dither_map(
            &original,
            16,
            1.0 / 16.0,
            &ChannelQuantization::new(4),
            None,
        )
        .snapshot("riemersma_map_flower_4");
    }
    #[test]
    fn riemersma_color_palette() {
        let img = read_flower();
        let palette_img = read_flower_palette();

        let palette = ColorPalette::new(RGB, palette_img.row(0).iter().copied(), BoundError);

        riemersma_dither_map(&img, 16, 1.0 / 16.0, &palette, None).snapshot("riemersma_palette");
    }
}
