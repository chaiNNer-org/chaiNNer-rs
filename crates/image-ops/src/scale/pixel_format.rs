use glam::{Vec2, Vec3A, Vec4};

pub trait PixelFormat: Send + Sync {
    /// Pixel type in the source image
    type InputPixel: Send + Sync + Copy;
    /// Pixel type in the destination image (usually the same as Input)
    type OutputPixel: Default + Send + Sync + Copy;
    /// Temporary struct for the pixel in floating-point
    type Accumulator: Send + Sync + Copy;

    /// Create new floating-point pixel
    fn new_acc() -> Self::Accumulator;
    /// Add new pixel with a given weight (first axis)
    fn add_pixel(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel);
    /// Add new pixel with a given weight (first axis)
    fn add_pixel_scaled(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32);
    /// Add bunch of accumulated pixels with a weight (second axis)
    fn add_acc_scaled(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32);
    /// Finalize, convert to output pixel format
    fn acc_to_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel;
    /// Finalize, convert to output pixel format
    fn acc_to_pixel_scaled(&self, acc: Self::Accumulator, coeff: f32) -> Self::OutputPixel;
}

pub struct FloatPixelFormat<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> Default for FloatPixelFormat<T> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> resize::PixelFormat for FloatPixelFormat<T>
where
    Self: PixelFormat,
{
    type InputPixel = <Self as PixelFormat>::InputPixel;

    type OutputPixel = <Self as PixelFormat>::OutputPixel;

    type Accumulator = <Self as PixelFormat>::Accumulator;

    #[inline(always)]
    fn new() -> Self::Accumulator {
        Self::new_acc()
    }

    #[inline(always)]
    fn add(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32) {
        self.add_pixel_scaled(acc, inp, coeff)
    }

    #[inline(always)]
    fn add_acc(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32) {
        Self::add_acc_scaled(acc, inp, coeff)
    }

    #[inline(always)]
    fn into_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel {
        self.acc_to_pixel(acc)
    }
}

impl PixelFormat for FloatPixelFormat<f32> {
    type InputPixel = f32;

    type OutputPixel = f32;

    type Accumulator = f32;

    #[inline(always)]
    fn new_acc() -> Self::Accumulator {
        Default::default()
    }

    #[inline(always)]
    fn add_pixel(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel) {
        *acc += inp;
    }

    #[inline(always)]
    fn add_pixel_scaled(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn add_acc_scaled(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn acc_to_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel {
        acc
    }

    #[inline(always)]
    fn acc_to_pixel_scaled(&self, acc: Self::Accumulator, coeff: f32) -> Self::OutputPixel {
        acc * coeff
    }
}
impl PixelFormat for FloatPixelFormat<Vec2> {
    type InputPixel = Vec2;

    type OutputPixel = Vec2;

    type Accumulator = Vec2;

    #[inline(always)]
    fn new_acc() -> Self::Accumulator {
        Default::default()
    }

    #[inline(always)]
    fn add_pixel(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel) {
        *acc += inp;
    }

    #[inline(always)]
    fn add_pixel_scaled(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn add_acc_scaled(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn acc_to_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel {
        acc
    }

    #[inline(always)]
    fn acc_to_pixel_scaled(&self, acc: Self::Accumulator, coeff: f32) -> Self::OutputPixel {
        acc * coeff
    }
}
impl PixelFormat for FloatPixelFormat<Vec3A> {
    type InputPixel = Vec3A;

    type OutputPixel = Vec3A;

    type Accumulator = Vec3A;

    #[inline(always)]
    fn new_acc() -> Self::Accumulator {
        Default::default()
    }

    #[inline(always)]
    fn add_pixel(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel) {
        *acc += inp;
    }

    #[inline(always)]
    fn add_pixel_scaled(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn add_acc_scaled(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn acc_to_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel {
        acc
    }

    #[inline(always)]
    fn acc_to_pixel_scaled(&self, acc: Self::Accumulator, coeff: f32) -> Self::OutputPixel {
        acc * coeff
    }
}
impl PixelFormat for FloatPixelFormat<Vec4> {
    type InputPixel = Vec4;

    type OutputPixel = Vec4;

    type Accumulator = Vec4;

    #[inline(always)]
    fn new_acc() -> Self::Accumulator {
        Default::default()
    }

    #[inline(always)]
    fn add_pixel(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel) {
        *acc += inp;
    }

    #[inline(always)]
    fn add_pixel_scaled(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn add_acc_scaled(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn acc_to_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel {
        acc
    }

    #[inline(always)]
    fn acc_to_pixel_scaled(&self, acc: Self::Accumulator, coeff: f32) -> Self::OutputPixel {
        acc * coeff
    }
}
impl PixelFormat for FloatPixelFormat<[f32; 3]> {
    type InputPixel = [f32; 3];

    type OutputPixel = [f32; 3];

    type Accumulator = Vec3A;

    #[inline(always)]
    fn new_acc() -> Self::Accumulator {
        Default::default()
    }

    #[inline(always)]
    fn add_pixel(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel) {
        *acc += Vec3A::from(inp);
    }

    #[inline(always)]
    fn add_pixel_scaled(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32) {
        *acc += Vec3A::from(inp) * coeff;
    }

    #[inline(always)]
    fn add_acc_scaled(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn acc_to_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel {
        acc.into()
    }

    #[inline(always)]
    fn acc_to_pixel_scaled(&self, acc: Self::Accumulator, coeff: f32) -> Self::OutputPixel {
        (acc * coeff).into()
    }
}
impl PixelFormat for FloatPixelFormat<[f32; 2]> {
    type InputPixel = [f32; 2];

    type OutputPixel = [f32; 2];

    type Accumulator = Vec2;

    #[inline(always)]
    fn new_acc() -> Self::Accumulator {
        Default::default()
    }

    #[inline(always)]
    fn add_pixel(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel) {
        *acc += Vec2::from(inp);
    }

    #[inline(always)]
    fn add_pixel_scaled(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32) {
        *acc += Vec2::from(inp) * coeff;
    }

    #[inline(always)]
    fn add_acc_scaled(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn acc_to_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel {
        acc.into()
    }

    #[inline(always)]
    fn acc_to_pixel_scaled(&self, acc: Self::Accumulator, coeff: f32) -> Self::OutputPixel {
        (acc * coeff).into()
    }
}
impl PixelFormat for FloatPixelFormat<[f32; 4]> {
    type InputPixel = [f32; 4];

    type OutputPixel = [f32; 4];

    type Accumulator = Vec4;

    #[inline(always)]
    fn new_acc() -> Self::Accumulator {
        Default::default()
    }

    #[inline(always)]
    fn add_pixel(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel) {
        *acc += Vec4::from(inp);
    }

    #[inline(always)]
    fn add_pixel_scaled(&self, acc: &mut Self::Accumulator, inp: Self::InputPixel, coeff: f32) {
        *acc += Vec4::from(inp) * coeff;
    }

    #[inline(always)]
    fn add_acc_scaled(acc: &mut Self::Accumulator, inp: Self::Accumulator, coeff: f32) {
        *acc += inp * coeff;
    }

    #[inline(always)]
    fn acc_to_pixel(&self, acc: Self::Accumulator) -> Self::OutputPixel {
        acc.into()
    }

    #[inline(always)]
    fn acc_to_pixel_scaled(&self, acc: Self::Accumulator, coeff: f32) -> Self::OutputPixel {
        (acc * coeff).into()
    }
}
