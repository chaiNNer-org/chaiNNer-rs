use glam::{Vec3A, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct Y(f32);
#[derive(Debug, Clone, Copy)]
pub struct Yuv(Vec3A);
#[derive(Debug, Clone, Copy)]
pub struct YuvA(Vec4);

const MAX_DIFF_Y: f32 = 3.0 / 255.0;
const MAX_DIFF_U: f32 = 7.0 / 255.0;
const MAX_DIFF_V: f32 = 6.0 / 255.0;
const MAX_DIFF_A: f32 = 1.0 / 255.0;

impl PartialEq for Y {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let diff = (self.0 - other.0).abs();
        diff <= MAX_DIFF_Y
    }
}
impl PartialEq for Yuv {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let diff = (self.0 - other.0).abs();
        diff.x <= MAX_DIFF_Y && diff.y <= MAX_DIFF_U && diff.z <= MAX_DIFF_V
    }
}
impl PartialEq for YuvA {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let diff = (self.0 - other.0).abs();
        diff.x <= MAX_DIFF_Y && diff.y <= MAX_DIFF_U && diff.z <= MAX_DIFF_V && diff.w <= MAX_DIFF_A
    }
}

#[inline]
fn rgb_to_yuv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let y = 0.299 * r + 0.587 * g + 0.114 * b;
    let u = -0.169 * r - 0.331 * g + 0.5 * b + 0.5;
    let v = 0.5 * r - 0.419 * g - 0.081 * b + 0.5;
    (y, u, v)
}

pub trait IntoYuv {
    type Output: Copy + PartialEq;

    fn into_yuv(self) -> Self::Output;
}

impl IntoYuv for f32 {
    type Output = Y;

    #[inline]
    fn into_yuv(self) -> Self::Output {
        Y(self)
    }
}
impl IntoYuv for Vec3A {
    type Output = Yuv;

    #[inline]
    fn into_yuv(self) -> Self::Output {
        let (y, u, v) = rgb_to_yuv(self.x, self.y, self.z);
        Yuv(Vec3A::new(y, u, v))
    }
}
impl IntoYuv for Vec4 {
    type Output = YuvA;

    #[inline]
    fn into_yuv(self) -> Self::Output {
        let (y, u, v) = rgb_to_yuv(self.x, self.y, self.z);
        YuvA(Vec4::new(y, u, v, self.w))
    }
}
