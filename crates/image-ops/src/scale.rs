use glam::{Vec2, Vec3A, Vec4};
use image_core::{Image, Size};
use resize::PixelFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Filter {
    Nearest,
    Linear,
    CubicCatrom,
    CubicMitchell,
    CubicBSpline,
    Lanczos3,
    Gauss,
}

struct FloatPixelFormat<T, G>
where
    G: CorrectGamma<T>,
{
    _marker: std::marker::PhantomData<T>,
    gamma: G,
}

impl<T, G> FloatPixelFormat<T, G>
where
    G: CorrectGamma<T>,
{
    pub fn new(gamma: G) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            gamma,
        }
    }
}

pub trait ResizePixel:
    Send + Sync + Copy + Default + std::ops::AddAssign + std::ops::Mul<f32, Output = Self>
{
}

impl ResizePixel for f32 {}
impl ResizePixel for Vec2 {}
impl ResizePixel for Vec3A {}
impl ResizePixel for Vec4 {}

impl<P: ResizePixel, G> PixelFormat for FloatPixelFormat<P, G>
where
    G: CorrectGamma<P> + Send + Sync,
{
    type InputPixel = P;

    type OutputPixel = P;

    type Accumulator = P;

    #[inline(always)]
    fn new() -> Self::Accumulator {
        Default::default()
    }

    #[inline(always)]
    fn add(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32) {
        *acc += self.gamma.to_linear(inp) * coeff;
    }

    #[inline(always)]
    fn add_acc(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn into_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel {
        self.gamma.from_linear(acc)
    }
}

pub trait CorrectGamma<T> {
    fn to_linear(&self, value: T) -> T;
    #[allow(clippy::wrong_self_convention)]
    fn from_linear(&self, value: T) -> T;
}

pub struct NoGammaCorrection;
impl<T> CorrectGamma<T> for NoGammaCorrection {
    #[inline(always)]
    fn to_linear(&self, value: T) -> T {
        value
    }
    #[inline(always)]
    fn from_linear(&self, value: T) -> T {
        value
    }
}

pub struct GammaCorrection;
impl GammaCorrection {
    const TO_LINEAR: f32 = 2.2;
    const TO_SRGB: f32 = 1.0 / 2.2;
}
impl CorrectGamma<f32> for GammaCorrection {
    #[inline(always)]
    fn to_linear(&self, value: f32) -> f32 {
        value.powf(Self::TO_LINEAR)
    }
    #[inline(always)]
    fn from_linear(&self, value: f32) -> f32 {
        value.powf(Self::TO_SRGB)
    }
}
impl CorrectGamma<Vec2> for GammaCorrection {
    #[inline(always)]
    fn to_linear(&self, value: Vec2) -> Vec2 {
        value.powf(Self::TO_LINEAR)
    }
    #[inline(always)]
    fn from_linear(&self, value: Vec2) -> Vec2 {
        value.powf(Self::TO_SRGB)
    }
}
impl CorrectGamma<Vec3A> for GammaCorrection {
    #[inline(always)]
    fn to_linear(&self, value: Vec3A) -> Vec3A {
        value.powf(Self::TO_LINEAR)
    }
    #[inline(always)]
    fn from_linear(&self, value: Vec3A) -> Vec3A {
        value.powf(Self::TO_SRGB)
    }
}
impl CorrectGamma<Vec4> for GammaCorrection {
    #[inline(always)]
    fn to_linear(&self, value: Vec4) -> Vec4 {
        let mut r = value.powf(Self::TO_LINEAR);
        r.w = value.w; // alpha is not gamma corrected
        r
    }
    #[inline(always)]
    fn from_linear(&self, value: Vec4) -> Vec4 {
        let mut r = value.powf(Self::TO_SRGB);
        r.w = value.w; // alpha is not gamma corrected
        r
    }
}

pub fn scale<P: ResizePixel>(
    img: &Image<P>,
    size: Size,
    filter: Filter,
    gamma: impl CorrectGamma<P> + Send + Sync,
) -> Result<Image<P>, resize::Error> {
    if size.is_empty() {
        return Ok(Image::new(size, Vec::new()));
    }

    let filter_type = match filter {
        Filter::Nearest => {
            // the nearest implementation isn't correct, so we use our own
            return Ok(nearest_neighbor(img, size));
        }
        Filter::Linear => resize::Type::Triangle,
        Filter::CubicCatrom => resize::Type::Catrom,
        Filter::CubicMitchell => resize::Type::Mitchell,
        Filter::CubicBSpline => {
            let filter = resize::Filter::new(
                // https://en.wikipedia.org/wiki/Mitchell%E2%80%93Netravali_filters#Implementations
                Box::new(|x| cubic_bc(1.0, 0.0, x)),
                2.0,
            );
            resize::Type::Custom(filter)
        }
        Filter::Lanczos3 => resize::Type::Lanczos3,
        Filter::Gauss => {
            let filter = resize::Filter::new(Box::new(|x| gaussian(x, 0.5)), 2.0);
            resize::Type::Custom(filter)
        }
    };

    let mut dest = Image::from_const(size, P::default());

    resize::Resizer::new(
        img.width(),
        img.height(),
        size.width,
        size.height,
        FloatPixelFormat::new(gamma),
        filter_type,
    )?
    .resize(img.data(), dest.data_mut())?;

    Ok(dest)
}

// Taken from
// https://github.com/PistonDevelopers/image/blob/2921cd7/src/imageops/sample.rs#L68
// TODO(Kagami): Could be optimized for known B and C, see e.g.
// https://github.com/sekrit-twc/zimg/blob/1a606c0/src/zimg/resize/filter.cpp#L149
#[inline(always)]
fn cubic_bc(b: f32, c: f32, x: f32) -> f32 {
    let a = x.abs();
    let k = if a < 1.0 {
        (12.0 - 9.0 * b - 6.0 * c) * a.powi(3)
            + (-18.0 + 12.0 * b + 6.0 * c) * a.powi(2)
            + (6.0 - 2.0 * b)
    } else if a < 2.0 {
        (-b - 6.0 * c) * a.powi(3)
            + (6.0 * b + 30.0 * c) * a.powi(2)
            + (-12.0 * b - 48.0 * c) * a
            + (8.0 * b + 24.0 * c)
    } else {
        0.0
    };
    k / 6.0
}

// Taken from: https://github.com/image-rs/image/blob/81b3fe66fba04b8b60ba79b3641826df22fca67e/src/imageops/sample.rs#L181
/// The Gaussian Function.
/// ```r``` is the standard deviation.
fn gaussian(x: f32, r: f32) -> f32 {
    ((2.0 * std::f32::consts::PI).sqrt() * r).recip() * (-x.powi(2) / (2.0 * r.powi(2))).exp()
}

fn nearest_neighbor<P: Clone>(src: &Image<P>, size: Size) -> Image<P> {
    if src.size() == size {
        return src.clone();
    }

    let src_size = src.size();
    let src = src.data();

    {
        // optimization for power-of-2 scaling factors, e.g. 2x, 4x
        let scale_up = size.width / src_size.width;
        if size == src_size.scale(scale_up as f64) && scale_up.is_power_of_two() {
            let shift = scale_up.trailing_zeros();

            let mut data = Vec::with_capacity(size.len());
            for y in 0..size.height {
                let src_y = y >> shift;
                let src_i = src_y * src_size.width;

                data.extend((0..size.width).map(move |x| {
                    let src_x = x >> shift;
                    src[src_i + src_x].clone()
                }));
            }

            return Image::new(size, data);
        }
    }

    // What is going on here? Okay, so this uses fixed point arithmetic (fixed)
    // to avoid floating point and divisions. Basic NN works like this:
    // We imagine that each pixel coordinate is at the center of the pixel and that center coordinate is then mapped to the src image. For the x coordinate this means:
    //
    //   src_x = round((x_index + 0.5) * src_width / width - 0.5)
    //         = floor((x_index + 0.5) * src_width / width)
    //
    // Let's define `k = src_width / width`.
    //
    //   src_x = floor((x_index + 0.5) * k)
    //         = floor(x_index * k + k/2)
    //
    // Now the fixed comes in. In fixed, `floor(x)` is just a bit shift, it's
    // super cheap.
    const SHIFT: i32 = 32;

    assert!(src_size.width <= i32::MAX as usize);
    assert!(src_size.height <= i32::MAX as usize);

    let k_x: u64 = ((src_size.width as u64) << SHIFT) / size.width as u64;
    let k_y: u64 = ((src_size.height as u64) << SHIFT) / size.height as u64;
    let k_x_half: u64 = k_x >> 1;
    let k_y_half: u64 = k_y >> 1;

    let mut data = Vec::with_capacity(size.len());
    for y in 0..(size.height as u64) {
        let src_y = ((y * k_y + k_y_half) >> SHIFT) as usize;
        let src_i = src_y * src_size.width;

        data.extend((0..(size.width as u64)).map(move |x| {
            let src_x = ((x * k_x + k_x_half) >> SHIFT) as usize;
            src[src_i + src_x].clone()
        }));
    }

    Image::new(size, data)
}

#[cfg(test)]
mod tests {
    use glam::Vec3A;
    use image_core::Size;
    use test_util::{data::read_portrait, snap::ImageSnapshot};

    use super::{GammaCorrection, NoGammaCorrection};

    fn small_portrait() -> image_core::Image<Vec3A> {
        let img = read_portrait();
        super::scale(
            &img,
            img.size().scale(0.5),
            super::Filter::Linear,
            NoGammaCorrection,
        )
        .unwrap()
    }

    #[test]
    fn scale_nearest() {
        let filter = super::Filter::Nearest;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(&original, new_size, filter, NoGammaCorrection).unwrap();
        nn.snapshot("resize_nearest_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(&original, new_size, filter, NoGammaCorrection).unwrap();
        nn.snapshot("resize_nearest_200");
    }

    #[test]
    fn scale_linear() {
        let filter = super::Filter::Linear;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(&original, new_size, filter, NoGammaCorrection).unwrap();
        nn.snapshot("resize_linear_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(&original, new_size, filter, NoGammaCorrection).unwrap();
        nn.snapshot("resize_linear_200");
    }

    #[test]
    fn scale_cubic_catrom() {
        let filter = super::Filter::CubicCatrom;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(&original, new_size, filter, NoGammaCorrection).unwrap();
        nn.snapshot("resize_cubic_catrom_4x");

        // https://github.com/chaiNNer-org/chaiNNer-rs/pull/20#issuecomment-1839525313
        // let original = read_portrait();
        // let new_size = Size::new(200, 200);
        // let nn = super::scale(&original, new_size, filter, NoGammaCorrection).unwrap();
        // nn.snapshot("resize_cubic_catrom_200");
    }

    #[test]
    fn scale_cubic_bspline() {
        let filter = super::Filter::CubicBSpline;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(&original, new_size, filter, NoGammaCorrection).unwrap();
        nn.snapshot("resize_cubic_bspline_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(&original, new_size, filter, NoGammaCorrection).unwrap();
        nn.snapshot("resize_cubic_bspline_200");
    }

    #[test]
    fn scale_cubic_mitchell() {
        let filter = super::Filter::CubicMitchell;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(&original, new_size, filter, NoGammaCorrection).unwrap();
        nn.snapshot("resize_cubic_mitchell_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(&original, new_size, filter, NoGammaCorrection).unwrap();
        nn.snapshot("resize_cubic_mitchell_200");
    }

    #[test]
    fn scale_lanczos3() {
        let filter = super::Filter::Lanczos3;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(&original, new_size, filter, NoGammaCorrection).unwrap();
        nn.snapshot("resize_lanczos3_4x");

        // https://github.com/chaiNNer-org/chaiNNer-rs/pull/20#issuecomment-1839525313
        // let original = read_portrait();
        // let new_size = Size::new(200, 200);
        // let nn = super::scale(&original, new_size, filter, NoGammaCorrection).unwrap();
        // nn.snapshot("resize_lanczos3_200");
    }

    #[test]
    fn scale_gauss() {
        let filter = super::Filter::Gauss;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(&original, new_size, filter, NoGammaCorrection).unwrap();
        nn.snapshot("resize_gauss_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(&original, new_size, filter, NoGammaCorrection).unwrap();
        nn.snapshot("resize_gauss_200");
    }

    #[test]
    fn scale_gamma() {
        let filter = super::Filter::CubicCatrom;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(&original, new_size, filter, GammaCorrection).unwrap();
        nn.snapshot("resize_gamma_correct_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(&original, new_size, filter, GammaCorrection).unwrap();
        nn.snapshot("resize_gamma_correct_200");
    }
}
