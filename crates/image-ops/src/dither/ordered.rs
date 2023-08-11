use image_core::{Image, NDimImage, Size};

use super::ChannelQuantization;

/// Creates a threshold map for ordered dithering.
///
/// `n` must be a power of 2.
///
/// Uses the algorithm presented here: https://bisqwit.iki.fi/story/howto/dither/jy/
fn create_threshold_map(n: usize) -> Image<f32> {
    assert!(n.is_power_of_two());
    let m = n.trailing_zeros();

    let mut result = Image::from_const(Size::new(n, n), 0.0);
    let map = result.data_mut();

    let area = (n * n) as f32;
    for i in 0..n {
        for j in 0..n {
            let mut v = 0;
            let xc = i ^ j;
            let yc = i;

            let mut bit = 0;
            for mask in (0..m).rev() {
                v |= ((yc >> mask) & 1) << (bit);
                bit += 1;
                v |= ((xc >> mask) & 1) << (bit);
                bit += 1;
            }

            map[i * n + j] = v as f32 / area;
        }
    }

    result
}

/// Stretch the image horizontally by a factor of `factor`. Each pixel is repeated `factor` times.
fn stretch_x<P: Copy + Default>(img: &Image<P>, factor: usize) -> Image<P> {
    if factor == 1 {
        return img.clone();
    }

    let mut result = Image::from_const(
        Size::new(img.width() * factor, img.height()),
        Default::default(),
    );
    let r_w = result.width();

    let src = img.data();
    let dst = result.data_mut();

    for y in 0..img.height() {
        let src_index = y * img.width();
        let dst_index = y * r_w;
        for x in 0..img.width() {
            let src_pixel = src[src_index + x];
            for i in 0..factor {
                dst[dst_index + x * factor + i] = src_pixel;
            }
        }
    }

    result
}

/// Tile the image horizontally to a new width.
fn tile_x<P: Copy + Default>(img: &Image<P>, new_width: usize) -> Image<P> {
    let mut result = Image::from_const(Size::new(new_width, img.height()), Default::default());

    let src_w = img.width();
    let dst_w = result.width();

    let src = img.data();
    let dst = result.data_mut();

    for y in 0..img.height() {
        let src_index = y * src_w;
        let dst_index = y * dst_w;
        for x in 0..new_width {
            dst[dst_index + x] = src[src_index + (x % src_w)];
        }
    }

    result
}

pub fn ordered_dither(img: &mut NDimImage, n: usize, quant: ChannelQuantization) {
    assert!(n.is_power_of_two());

    if quant.per_channel() == 2 {
        return binary_ordered_dither(img, n, 0.5);
    }

    let f = (quant.per_channel() - 1) as f32;

    // The idea here is to make the threshold map the same width as a row of the image.
    // This allows us to zip the current threshold row with the current image row, which
    // gets rid of the inner channel loop, which makes the code around 25% faster.
    let threshold_map = tile_x(
        &stretch_x(&create_threshold_map(n), img.channels()),
        img.width() * img.channels(),
    );
    let n_mask = n - 1;

    let shape = img.shape();
    let data = img.data_mut();

    for y in 0..shape.height {
        let threshold_row = threshold_map.row(y & n_mask);
        let data_row =
            &mut data[(y * shape.width * shape.channels)..((y + 1) * shape.width * shape.channels)];
        assert_eq!(threshold_row.len(), data_row.len());

        for (data, threshold) in data_row.iter_mut().zip(threshold_row.iter()) {
            *data = (*data * f + threshold).floor() / f;
        }
    }
}

fn binary_ordered_dither(img: &mut NDimImage, n: usize, bin_threshold: f32) {
    assert!(n.is_power_of_two());

    // Same idea as in the regular ordered dither, but we get even more out of it.
    // The inner channel loop prevented effective vectorization.
    // Binary ordered dithering is about 5x faster with this trick.
    let threshold_map = tile_x(
        &stretch_x(
            &create_threshold_map(n).map(|f| bin_threshold + 0.5 - f),
            img.channels(),
        ),
        img.width() * img.channels(),
    );
    let n_mask = n - 1;

    let shape = img.shape();
    let data = img.data_mut();

    for y in 0..shape.height {
        let threshold_row = threshold_map.row(y & n_mask);
        let data_row =
            &mut data[(y * shape.width * shape.channels)..((y + 1) * shape.width * shape.channels)];
        assert_eq!(threshold_row.len(), data_row.len());

        for (data, threshold) in data_row.iter_mut().zip(threshold_row.iter()) {
            // This is equivalent to:
            //     *data = (*data * f + threshold).floor() / f;
            // Since `data` ∈ [0, 1], `threshold` ∈ [0, 1), and `f = 1`, it simplifies to what's below.
            *data = if *data >= *threshold { 1.0 } else { 0.0 };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_util::{data::read_flower, snap::ImageSnapshot};

    #[test]
    fn ordered_dither_channels() {
        let mut img = read_flower().into();
        ordered_dither(&mut img, 4, ChannelQuantization::new(4));
        img.snapshot("ordered_4_4x4");

        let mut img = read_flower().into();
        ordered_dither(&mut img, 16, ChannelQuantization::new(4));
        img.snapshot("ordered_4_16x16");

        let mut img = read_flower().into();
        ordered_dither(&mut img, 64, ChannelQuantization::new(4));
        img.snapshot("ordered_4_64x64");

        let mut img = read_flower().into();
        ordered_dither(&mut img, 4, ChannelQuantization::new(2));
        img.snapshot("ordered_2_4x4");
    }
}
