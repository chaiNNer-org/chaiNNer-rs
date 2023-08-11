use ahash::AHashSet;
use image_core::{
    util::{slice_as_chunks, vec_into_flattened},
    NDimImage, NDimView, Shape,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ExtractionError {
    TooManyColors {
        max_colors: usize,
        actual_colors: usize,
    },
    UnsupportedChannels {
        channels: usize,
    },
}

fn sort_colors<const N: usize>(colors: &mut [[f32; N]], key_fn: impl Fn(&[f32; N]) -> f32) {
    colors.sort_unstable_by(|a, b| key_fn(a).total_cmp(&key_fn(b)))
}

pub fn extract_unique_const<const N: usize>(
    src: impl IntoIterator<Item = [f32; N]>,
    max_colors: usize,
) -> Result<Vec<[f32; N]>, ExtractionError> {
    assert_ne!(N, 0);

    let set: AHashSet<[u32; N]> = src.into_iter().map(|p| p.map(f32::to_bits)).collect();

    if set.len() > max_colors {
        return Err(ExtractionError::TooManyColors {
            max_colors,
            actual_colors: set.len(),
        });
    }

    let mut vec: Vec<[f32; N]> = set.into_iter().map(|p| p.map(f32::from_bits)).collect();

    fn luminance(r: f32, g: f32, b: f32) -> f32 {
        // Since the color values are likely sRGB, we will approximate 2.2 gamma by squaring the values.
        r * r * 0.2126 + g * g * 0.7152 + b * b * 0.0722
    }

    match N {
        1 => sort_colors(&mut vec, |f| f[0]),
        3 => sort_colors(&mut vec, |f| luminance(f[0], f[1], f[2])),
        4 => sort_colors(&mut vec, |f| {
            // we want values to sorted by alpha first, so we give it a large weight
            luminance(f[0], f[1], f[2]) + f[3] * 10.0
        }),
        _ => sort_colors(&mut vec, |f| f.iter().sum()),
    }

    Ok(vec)
}

pub fn extract_unique_ndim_const<const N: usize>(
    src: NDimView,
    max_colors: usize,
) -> Result<Vec<[f32; N]>, ExtractionError> {
    assert_ne!(N, 0);

    if src.channels() != N {
        return Err(ExtractionError::UnsupportedChannels {
            channels: src.channels(),
        });
    }

    let (pixels, rest) = slice_as_chunks::<f32, N>(src.data());
    assert!(rest.is_empty());

    extract_unique_const(pixels.iter().copied(), max_colors)
}

pub fn extract_unique_ndim(src: NDimView, max_colors: usize) -> Result<NDimImage, ExtractionError> {
    fn extract<const N: usize>(
        src: NDimView,
        max_colors: usize,
    ) -> Result<NDimImage, ExtractionError> {
        let colors: Vec<[f32; N]> = extract_unique_ndim_const::<N>(src, max_colors)?;

        let shape = Shape::new(colors.len(), 1, N);
        let data = vec_into_flattened(colors);
        Ok(NDimImage::new(shape, data))
    }

    match src.channels() {
        1 => extract::<1>(src, max_colors),
        2 => extract::<2>(src, max_colors),
        3 => extract::<3>(src, max_colors),
        4 => extract::<4>(src, max_colors),
        _ => Err(ExtractionError::UnsupportedChannels {
            channels: src.channels(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_util::data::read_flower_palette;

    #[test]
    fn nearest_neighbor() {
        let original: NDimImage = read_flower_palette().into();

        let palette = extract_unique_ndim(original.view(), 256).unwrap();
        assert_eq!(palette.shape(), original.shape());
    }
}
