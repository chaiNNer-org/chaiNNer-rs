use glam::{Vec2, Vec3, Vec3A, Vec4};
use image_core::{Image, NDimImage};
use rstar::{primitives::GeomWithData, Point, RTree};

use super::Pixel;

pub trait ErrorCombinator<P> {
    fn combine_error(&self, color: P, error: P) -> P;
}

pub trait ColorLookup<P> {
    type Nearest;

    fn get_nearest_color(&self, color: P) -> Self::Nearest;

    fn get_error(&self, color: P, nearest: Self::Nearest) -> P;
}

pub trait Quantizer<P, N>: ColorLookup<P, Nearest = N> + ErrorCombinator<P> {}
impl<P, N, T: ColorLookup<P, Nearest = N> + ErrorCombinator<P>> Quantizer<P, N> for T {}

pub struct BoundError;
impl ErrorCombinator<f32> for BoundError {
    #[inline(always)]
    fn combine_error(&self, color: f32, error: f32) -> f32 {
        (color + error).clamp(0.0, 1.0)
    }
}
macro_rules! impl_bound_error_vec {
    ($t:ident) => {
        impl ErrorCombinator<$t> for BoundError {
            #[inline(always)]
            fn combine_error(&self, color: $t, error: $t) -> $t {
                (color + error).clamp($t::ZERO, $t::ONE)
            }
        }
    };
}
impl_bound_error_vec!(Vec2);
impl_bound_error_vec!(Vec3);
impl_bound_error_vec!(Vec3A);
impl_bound_error_vec!(Vec4);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelQuantization {
    per_channel: usize,
    factor: f32,
    factor_inv: f32,
}

impl ChannelQuantization {
    pub fn new(per_channel: usize) -> Self {
        assert!(per_channel >= 2);

        let factor = (per_channel - 1) as f32;
        Self {
            per_channel,
            factor,
            factor_inv: 1.0 / factor,
        }
    }

    pub fn per_channel(&self) -> usize {
        self.per_channel
    }
}

impl<P: Pixel> ErrorCombinator<P> for ChannelQuantization {
    #[inline(always)]
    fn combine_error(&self, mut color: P, error: P) -> P {
        // Since the quantization error is always less than 1/number_of_colors, we don't have to clamp the result.
        color += error;
        color
    }
}

impl ColorLookup<f32> for ChannelQuantization {
    type Nearest = f32;

    #[inline(always)]
    fn get_nearest_color(&self, color: f32) -> Self::Nearest {
        ((color * self.factor + 0.5).floor() * self.factor_inv).clamp(0.0, 1.0)
    }

    #[inline(always)]
    fn get_error(&self, color: f32, nearest: Self::Nearest) -> f32 {
        color - nearest
    }
}

macro_rules! impl_channels_vec {
    ($t:ident) => {
        impl ColorLookup<$t> for ChannelQuantization {
            type Nearest = $t;

            #[inline(always)]
            fn get_nearest_color(&self, color: $t) -> Self::Nearest {
                ((color * self.factor).round() * self.factor_inv).clamp($t::ZERO, $t::ONE)
            }

            #[inline(always)]
            fn get_error(&self, color: $t, nearest: Self::Nearest) -> $t {
                color - nearest
            }
        }
    };
}
impl_channels_vec!(Vec2);
impl_channels_vec!(Vec3);
impl_channels_vec!(Vec3A);
impl_channels_vec!(Vec4);

pub trait ColorSpace<P> {
    type Coord: Point<Scalar = f32>;

    fn get_coordinate(&self, color: P) -> Self::Coord;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RGB;
impl ColorSpace<f32> for RGB {
    type Coord = [f32; 1];

    fn get_coordinate(&self, color: f32) -> Self::Coord {
        [color]
    }
}
impl<const N: usize> ColorSpace<[f32; N]> for RGB {
    type Coord = [f32; N];

    fn get_coordinate(&self, color: [f32; N]) -> Self::Coord {
        color
    }
}

macro_rules! impl_srgb_into {
    ($t:ty, $n:literal) => {
        impl ColorSpace<$t> for RGB {
            type Coord = [f32; $n];

            fn get_coordinate(&self, color: $t) -> Self::Coord {
                color.into()
            }
        }
    };
}
impl_srgb_into!(Vec2, 2);
impl_srgb_into!(Vec3, 3);
impl_srgb_into!(Vec3A, 3);
impl_srgb_into!(Vec4, 4);

#[derive(Debug, Clone)]
enum Lookup<G: Point<Scalar = f32>, P> {
    Linear(Vec<GeomWithData<G, P>>),
    Tree(RTree<GeomWithData<G, P>>),
}
impl<G: Point<Scalar = f32>, P: Clone> Lookup<G, P> {
    pub fn new(colors: Vec<GeomWithData<G, P>>) -> Self {
        if colors.len() < 300 {
            // linear lookup is really fast for small palettes
            Self::Linear(colors)
        } else {
            Self::Tree(RTree::bulk_load(colors))
        }
    }

    fn distance_2(a: &G, b: &G) -> f32 {
        (0..G::DIMENSIONS)
            .map(|i| a.nth(i) - b.nth(i))
            .map(|d| d * d)
            .sum()
    }

    pub fn get_nearest_color(&self, color: G) -> P {
        match self {
            Self::Linear(colors) => {
                let mut nearest = &colors[0];
                let mut nearest_dist = Self::distance_2(nearest.geom(), &color);
                for c in colors.iter().skip(1) {
                    let dist = Self::distance_2(c.geom(), &color);
                    if dist < nearest_dist {
                        nearest = c;
                        nearest_dist = dist;
                    }
                }
                nearest.data.clone()
            }
            Self::Tree(colors) => colors
                .nearest_neighbor(&color)
                .map(|c| c.data.clone())
                .expect("palette to not be empty"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ColorPalette<P: Clone, C: ColorSpace<P>, E> {
    colorspace: C,
    lookup: Lookup<C::Coord, P>,
    error: E,
}

impl<P: Copy, C: ColorSpace<P>, E: ErrorCombinator<P>> ColorPalette<P, C, E> {
    pub fn new(colorspace: C, colors: impl IntoIterator<Item = P>, error: E) -> Self {
        let colors: Vec<GeomWithData<C::Coord, P>> = colors
            .into_iter()
            .map(|color| {
                let coord = colorspace.get_coordinate(color);
                GeomWithData::new(coord, color)
            })
            .collect();

        assert!(
            !colors.is_empty(),
            "palette must contain at least one color"
        );
        let lookup = Lookup::new(colors);

        Self {
            colorspace,
            lookup,
            error,
        }
    }
}

impl<P, C: ColorSpace<P>, E> ColorLookup<P> for ColorPalette<P, C, E>
where
    P: Copy + std::ops::Sub<Output = P>,
{
    type Nearest = P;

    fn get_nearest_color(&self, color: P) -> Self::Nearest {
        let coord = self.colorspace.get_coordinate(color);
        self.lookup.get_nearest_color(coord)
    }

    #[inline(always)]
    fn get_error(&self, color: P, nearest: Self::Nearest) -> P {
        color - nearest
    }
}

impl<P: Clone, C: ColorSpace<P>, E: ErrorCombinator<P>> ErrorCombinator<P>
    for ColorPalette<P, C, E>
{
    #[inline(always)]
    fn combine_error(&self, color: P, error: P) -> P {
        self.error.combine_error(color, error)
    }
}

pub fn quantize<P: Clone>(img: &mut Image<P>, quant: &impl ColorLookup<P, Nearest = P>) {
    for p in img.data_mut() {
        *p = quant.get_nearest_color(p.clone());
    }
}
pub fn quantize_ndim(img: &mut NDimImage, quant: ChannelQuantization) {
    if quant.per_channel() == 2 {
        for p in img.data_mut() {
            *p = if *p >= 0.5 { 1.0 } else { 0.0 };
        }
    } else {
        let f = (quant.per_channel() - 1) as f32;
        let f_inv = 1_f32 / f;
        for p in img.data_mut() {
            *p = (*p * f + 0.5).floor() * f_inv;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dither::ChannelQuantization;

    use super::*;
    use test_util::{data::read_flower, snap::ImageSnapshot};

    #[test]
    fn quantize_image() {
        let mut img = read_flower();
        quantize(&mut img, &ChannelQuantization::new(4));
        img.snapshot("quantize_4");
    }
    #[test]
    fn quantize_ndim_image() {
        let mut img: NDimImage = read_flower().into();
        quantize_ndim(&mut img, ChannelQuantization::new(4));
        img.snapshot("quantize_ndim_4");
    }
}
