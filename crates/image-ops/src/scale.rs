use glam::{Vec2, Vec3A, Vec4};
use image_core::{Image, ImageView, Size};
pub use resize::PixelFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Filter {
    Nearest,
    Box,
    Linear,
    Hermite,
    CubicCatrom,
    CubicMitchell,
    CubicBSpline,
    Hamming,
    Hann,
    Lanczos3,
    Lagrange,
    Gauss,
}

impl From<Filter> for resize::Type {
    fn from(filter: Filter) -> Self {
        #[inline]
        fn sinc(x: f32) -> f32 {
            if x == 0.0 {
                1.0
            } else {
                x.sin() / x
            }
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

        fn lagrange(x: f32, support: f32) -> f32 {
            let x = x.abs();
            if x > support {
                return 0.0;
            }

            // Taken from
            // https://github.com/ImageMagick/ImageMagick/blob/e8b7974e8756fb278ec85d896065a1b96ed85af9/MagickCore/resize.c#L406
            let order = (2.0 * support) as isize;
            let n = (support + x) as isize;
            let mut value = 1.0;
            for i in 0..order {
                let d = (n - i) as f32;
                if d != 0.0 {
                    value *= (d - x) / d;
                }
            }
            value
        }

        match filter {
            Filter::Nearest => resize::Type::Point,
            Filter::Box => {
                let filter =
                    resize::Filter::new(Box::new(|x| if x.abs() <= 0.5 { 1.0 } else { 0.0 }), 1.0);
                resize::Type::Custom(filter)
            }
            Filter::Linear => resize::Type::Triangle,
            Filter::Hermite => {
                let filter = resize::Filter::new(Box::new(|x| cubic_bc(0.0, 0.0, x)), 1.0);
                resize::Type::Custom(filter)
            }
            Filter::CubicCatrom => resize::Type::Catrom,
            Filter::CubicMitchell => resize::Type::Mitchell,
            Filter::CubicBSpline => resize::Type::BSpline,
            Filter::Hamming => {
                let filter = resize::Filter::new(
                    Box::new(|x| {
                        let x = x.abs() * std::f32::consts::PI;
                        sinc(x) * (0.54 + 0.46 * x.cos())
                    }),
                    1.0,
                );
                resize::Type::Custom(filter)
            }
            Filter::Hann => {
                let filter = resize::Filter::new(
                    Box::new(|x| {
                        let x = x.abs() * std::f32::consts::PI;
                        sinc(x) * (0.5 + 0.5 * x.cos())
                    }),
                    1.0,
                );
                resize::Type::Custom(filter)
            }
            Filter::Lanczos3 => resize::Type::Lanczos3,
            Filter::Lagrange => {
                let filter = resize::Filter::new(Box::new(|x| lagrange(x, 2.0)), 2.0);
                resize::Type::Custom(filter)
            }
            Filter::Gauss => resize::Type::Gaussian,
        }
    }
}

pub struct FloatPixelFormat<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> FloatPixelFormat<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl PixelFormat for FloatPixelFormat<f32> {
    type InputPixel = f32;

    type OutputPixel = f32;

    type Accumulator = f32;

    #[inline(always)]
    fn new() -> Self::Accumulator {
        Default::default()
    }

    #[inline(always)]
    fn add(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn add_acc(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn into_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel {
        acc
    }
}
impl PixelFormat for FloatPixelFormat<Vec2> {
    type InputPixel = Vec2;

    type OutputPixel = Vec2;

    type Accumulator = Vec2;

    #[inline(always)]
    fn new() -> Self::Accumulator {
        Default::default()
    }

    #[inline(always)]
    fn add(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn add_acc(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn into_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel {
        acc
    }
}
impl PixelFormat for FloatPixelFormat<Vec3A> {
    type InputPixel = Vec3A;

    type OutputPixel = Vec3A;

    type Accumulator = Vec3A;

    #[inline(always)]
    fn new() -> Self::Accumulator {
        Default::default()
    }

    #[inline(always)]
    fn add(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn add_acc(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn into_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel {
        acc
    }
}
impl PixelFormat for FloatPixelFormat<Vec4> {
    type InputPixel = Vec4;

    type OutputPixel = Vec4;

    type Accumulator = Vec4;

    #[inline(always)]
    fn new() -> Self::Accumulator {
        Default::default()
    }

    #[inline(always)]
    fn add(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn add_acc(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn into_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel {
        acc
    }
}
impl PixelFormat for FloatPixelFormat<[f32; 3]> {
    type InputPixel = [f32; 3];

    type OutputPixel = [f32; 3];

    type Accumulator = Vec3A;

    #[inline(always)]
    fn new() -> Self::Accumulator {
        Default::default()
    }

    #[inline(always)]
    fn add(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32) {
        *acc += Vec3A::from(inp) * coeff;
    }

    #[inline(always)]
    fn add_acc(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn into_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel {
        acc.into()
    }
}
impl PixelFormat for FloatPixelFormat<[f32; 2]> {
    type InputPixel = [f32; 2];

    type OutputPixel = [f32; 2];

    type Accumulator = Vec2;

    #[inline(always)]
    fn new() -> Self::Accumulator {
        Default::default()
    }

    #[inline(always)]
    fn add(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32) {
        *acc += Vec2::from(inp) * coeff;
    }

    #[inline(always)]
    fn add_acc(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn into_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel {
        acc.into()
    }
}
impl PixelFormat for FloatPixelFormat<[f32; 4]> {
    type InputPixel = [f32; 4];

    type OutputPixel = [f32; 4];

    type Accumulator = Vec4;

    #[inline(always)]
    fn new() -> Self::Accumulator {
        Default::default()
    }

    #[inline(always)]
    fn add(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32) {
        *acc += Vec4::from(inp) * coeff;
    }

    #[inline(always)]
    fn add_acc(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn into_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel {
        acc.into()
    }
}

pub fn scale<P>(img: ImageView<P>, size: Size, filter: Filter) -> Result<Image<P>, resize::Error>
where
    P: Clone + Default,
    FloatPixelFormat<P>: PixelFormat<InputPixel = P, OutputPixel = P>,
{
    if size.is_empty() {
        return Ok(Image::new(size, Vec::new()));
    }

    let filter_type = match filter {
        Filter::Nearest => {
            // the nearest implementation isn't correct, so we use our own
            return Ok(nearest_neighbor(img, size));
        }
        _ => filter.into(),
    };

    let mut dest = Image::from_const(size, P::default());

    resize::Resizer::new(
        img.width(),
        img.height(),
        size.width,
        size.height,
        FloatPixelFormat::new(),
        filter_type,
    )?
    .resize(img.data(), dest.data_mut())?;

    Ok(dest)
}

fn nearest_neighbor<P: Clone>(src: ImageView<P>, size: Size) -> Image<P> {
    if src.size() == size {
        return src.into_owned();
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

    fn small_portrait() -> image_core::Image<Vec3A> {
        let img = read_portrait();
        super::scale(img.view(), img.size().scale(0.5), super::Filter::Linear).unwrap()
    }

    #[test]
    fn scale_nearest() {
        let filter = super::Filter::Nearest;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_nearest_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_nearest_200");
    }

    #[test]
    fn scale_box() {
        let filter = super::Filter::Box;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_box_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_box_200");
    }

    #[test]
    fn scale_linear() {
        let filter = super::Filter::Linear;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_linear_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_linear_200");
    }

    #[test]
    fn scale_hermite() {
        let filter = super::Filter::Hermite;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_hermite_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_hermite_200");
    }

    #[test]
    fn scale_cubic_catrom() {
        let filter = super::Filter::CubicCatrom;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_cubic_catrom_4x");

        // https://github.com/chaiNNer-org/chaiNNer-rs/pull/20#issuecomment-1839525313
        // let original = read_portrait();
        // let new_size = Size::new(200, 200);
        // let nn = super::scale(original.view(), new_size, filter).unwrap();
        // nn.snapshot("resize_cubic_catrom_200");
    }

    #[test]
    fn scale_cubic_bspline() {
        let filter = super::Filter::CubicBSpline;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_cubic_bspline_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_cubic_bspline_200");
    }

    #[test]
    fn scale_cubic_mitchell() {
        let filter = super::Filter::CubicMitchell;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_cubic_mitchell_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_cubic_mitchell_200");
    }

    #[test]
    fn scale_hamming() {
        let filter = super::Filter::Hamming;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_hamming_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_hamming_200");
    }

    #[test]
    fn scale_hann() {
        let filter = super::Filter::Hann;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_hann_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_hann_200");
    }

    #[test]
    fn scale_lanczos3() {
        let filter = super::Filter::Lanczos3;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_lanczos3_4x");

        // https://github.com/chaiNNer-org/chaiNNer-rs/pull/20#issuecomment-1839525313
        // let original = read_portrait();
        // let new_size = Size::new(200, 200);
        // let nn = super::scale(original.view(), new_size, filter).unwrap();
        // nn.snapshot("resize_lanczos3_200");
    }

    #[test]
    fn scale_gauss() {
        let filter = super::Filter::Gauss;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_gauss_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_gauss_200");
    }

    #[test]
    fn scale_lagrange() {
        let filter = super::Filter::Lagrange;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_lagrange_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_lagrange_200");
    }
}
