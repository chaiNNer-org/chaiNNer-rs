use std::borrow::Cow;

use glam::{Vec2, Vec3, Vec3A, Vec4};

use crate::util::{slice_as_chunks, vec_into_chunks, vec_into_flattened, vec_try_transmute};

pub trait Components {
    const COMPONENTS: usize;
}
impl Components for f32 {
    const COMPONENTS: usize = 1;
}
impl<const N: usize> Components for [f32; N] {
    const COMPONENTS: usize = N;
}
impl Components for Vec2 {
    const COMPONENTS: usize = 2;
}
impl Components for Vec3 {
    const COMPONENTS: usize = 3;
}
impl Components for Vec3A {
    const COMPONENTS: usize = 3;
}
impl Components for Vec4 {
    const COMPONENTS: usize = 4;
}

pub trait Flatten: Components + Sized {
    fn flatten_pixels(vec: Vec<Self>) -> Vec<f32>;
}

impl Flatten for f32 {
    fn flatten_pixels(vec: Vec<Self>) -> Vec<f32> {
        vec
    }
}
impl<const N: usize> Flatten for [f32; N] {
    fn flatten_pixels(vec: Vec<Self>) -> Vec<f32> {
        vec_into_flattened(vec)
    }
}
impl Flatten for Vec2 {
    fn flatten_pixels(vec: Vec<Self>) -> Vec<f32> {
        let vec: Vec<_> = vec.into_iter().map(|x| x.into()).collect();
        vec_into_flattened(vec)
    }
}
impl Flatten for Vec3 {
    fn flatten_pixels(vec: Vec<Self>) -> Vec<f32> {
        let vec: Vec<_> = vec.into_iter().map(|x| x.into()).collect();
        vec_into_flattened(vec)
    }
}
impl Flatten for Vec3A {
    fn flatten_pixels(vec: Vec<Self>) -> Vec<f32> {
        let vec: Vec<_> = vec.into_iter().map(|x| x.into()).collect();
        vec_into_flattened(vec)
    }
}
impl Flatten for Vec4 {
    fn flatten_pixels(vec: Vec<Self>) -> Vec<f32> {
        // we might be able to avoid the copy here by casting the vec directly.
        let vec = match unsafe { vec_try_transmute::<Vec4, [f32; 4]>(vec) } {
            Ok(vec) => return vec_into_flattened(vec),
            Err(vec) => vec,
        };

        // slow copy *shouldn't* happen
        let vec: Vec<_> = vec.into_iter().map(|x| x.into()).collect();
        vec_into_flattened(vec)
    }
}

pub struct UnsupportedChannel {
    pub supported: &'static [usize],
}

fn iter_rg<T>(
    flat: &[f32],
    channels: usize,
    f: impl Fn([f32; 2]) -> T,
) -> Result<Vec<T>, UnsupportedChannel> {
    match channels {
        1 => Ok(flat.iter().map(|g| f([*g, *g])).collect()),
        2 => {
            let (chunks, rest) = slice_as_chunks::<_, 2>(flat);
            assert!(rest.is_empty());
            Ok(chunks.iter().map(|[x, y]| f([*x, *y])).collect())
        }
        _ => Err(UnsupportedChannel { supported: &[1, 2] }),
    }
}
fn iter_rgb<T>(
    flat: &[f32],
    channels: usize,
    f: impl Fn([f32; 3]) -> T,
) -> Result<Vec<T>, UnsupportedChannel> {
    match channels {
        1 => Ok(flat.iter().map(|g| f([*g, *g, *g])).collect()),
        3 => {
            let (chunks, rest) = slice_as_chunks::<_, 3>(flat);
            assert!(rest.is_empty());
            Ok(chunks.iter().map(|[x, y, z]| f([*x, *y, *z])).collect())
        }
        _ => Err(UnsupportedChannel { supported: &[1, 3] }),
    }
}
fn iter_rgba<T>(
    flat: &[f32],
    channels: usize,
    f: impl Fn([f32; 4]) -> T,
) -> Result<Vec<T>, UnsupportedChannel> {
    match channels {
        1 => Ok(flat.iter().map(|g| f([*g, *g, *g, 1.])).collect()),
        3 => {
            let (chunks, rest) = slice_as_chunks::<_, 3>(flat);
            assert!(rest.is_empty());
            Ok(chunks.iter().map(|[x, y, z]| f([*x, *y, *z, 1.])).collect())
        }
        4 => {
            let (chunks, rest) = slice_as_chunks::<_, 4>(flat);
            assert!(rest.is_empty());
            Ok(chunks
                .iter()
                .map(|[x, y, z, w]| f([*x, *y, *z, *w]))
                .collect())
        }
        _ => Err(UnsupportedChannel {
            supported: &[1, 3, 4],
        }),
    }
}

pub trait FromFlat: Components + Sized
where
    Self: Clone,
    [Self]: std::borrow::ToOwned<Owned = Vec<Self>>,
{
    fn from_flat_slice(
        slice: &[f32],
        channels: usize,
    ) -> Result<Cow<'_, [Self]>, UnsupportedChannel>;
    fn from_flat_vec(vec: Vec<f32>, channels: usize) -> Result<Vec<Self>, UnsupportedChannel> {
        Ok(Self::from_flat_slice(&vec, channels)?.into_owned())
    }
}

impl FromFlat for f32 {
    fn from_flat_slice(
        slice: &[f32],
        channels: usize,
    ) -> Result<Cow<'_, [Self]>, UnsupportedChannel> {
        if channels == 1 {
            return Ok(Cow::Borrowed(slice));
        }
        Err(UnsupportedChannel { supported: &[1] })
    }
    fn from_flat_vec(vec: Vec<f32>, channels: usize) -> Result<Vec<Self>, UnsupportedChannel> {
        if channels == 1 {
            return Ok(vec);
        }
        Err(UnsupportedChannel { supported: &[1] })
    }
}
impl<const N: usize> FromFlat for [f32; N] {
    fn from_flat_slice(
        slice: &[f32],
        channels: usize,
    ) -> Result<Cow<'_, [Self]>, UnsupportedChannel> {
        if channels == N {
            let (chunks, rest) = slice_as_chunks(slice);
            assert!(rest.is_empty());
            return Ok(Cow::Borrowed(chunks));
        }
        Err(UnsupportedChannel { supported: &[N] })
    }
    fn from_flat_vec(vec: Vec<f32>, channels: usize) -> Result<Vec<Self>, UnsupportedChannel> {
        if channels == N {
            return Ok(vec_into_chunks(vec));
        }
        Err(UnsupportedChannel { supported: &[N] })
    }
}
impl FromFlat for Vec2 {
    fn from_flat_slice(
        slice: &[f32],
        channels: usize,
    ) -> Result<Cow<'_, [Self]>, UnsupportedChannel> {
        let vec = iter_rg(slice, channels, |a| a.into())?;
        Ok(Cow::Owned(vec))
    }
}
impl FromFlat for Vec3 {
    fn from_flat_slice(
        slice: &[f32],
        channels: usize,
    ) -> Result<Cow<'_, [Self]>, UnsupportedChannel> {
        let vec = iter_rgb(slice, channels, |a| a.into())?;
        Ok(Cow::Owned(vec))
    }
}
impl FromFlat for Vec3A {
    fn from_flat_slice(
        slice: &[f32],
        channels: usize,
    ) -> Result<Cow<'_, [Self]>, UnsupportedChannel> {
        let vec = iter_rgb(slice, channels, |a| a.into())?;
        Ok(Cow::Owned(vec))
    }
}
impl FromFlat for Vec4 {
    fn from_flat_slice(
        slice: &[f32],
        channels: usize,
    ) -> Result<Cow<'_, [Self]>, UnsupportedChannel> {
        let vec = iter_rgba(slice, channels, |a| a.into())?;
        Ok(Cow::Owned(vec))
    }
}

pub trait ClipFloat {
    fn clip(self, min: f32, max: f32) -> Self;
}

impl ClipFloat for f32 {
    fn clip(self, min: f32, max: f32) -> Self {
        self.clamp(min, max)
    }
}
impl<const N: usize> ClipFloat for [f32; N] {
    fn clip(self, min: f32, max: f32) -> Self {
        self.map(|x| x.clamp(min, max))
    }
}
impl ClipFloat for Vec2 {
    fn clip(self, min: f32, max: f32) -> Self {
        self.clamp(Self::splat(min), Self::splat(max))
    }
}
impl ClipFloat for Vec3 {
    fn clip(self, min: f32, max: f32) -> Self {
        self.clamp(Self::splat(min), Self::splat(max))
    }
}
impl ClipFloat for Vec3A {
    fn clip(self, min: f32, max: f32) -> Self {
        self.clamp(Self::splat(min), Self::splat(max))
    }
}
impl ClipFloat for Vec4 {
    fn clip(self, min: f32, max: f32) -> Self {
        self.clamp(Self::splat(min), Self::splat(max))
    }
}
