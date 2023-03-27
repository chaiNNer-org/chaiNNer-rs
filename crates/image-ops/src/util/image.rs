use std::ops::{Deref, DerefMut};

use image_core::{Image, Size};

pub fn from_const<P: Clone>(size: Size, c: P, out: Option<Image<P>>) -> Image<P> {
    if let Some(mut out) = out {
        assert_eq!(out.size(), size);
        out.fill(c);
        out
    } else {
        Image::from_const(size, c)
    }
}

#[allow(unused)]
pub fn from_image<P: Copy>(img: &Image<P>, out: Option<Image<P>>) -> Image<P> {
    if let Some(mut out) = out {
        assert_eq!(out.size(), img.size());
        out.data_mut().copy_from_slice(img.data());
        out
    } else {
        img.clone()
    }
}

pub enum ImageCow<'a, P> {
    Owned(Image<P>),
    Borrowed(&'a mut Image<P>),
}

impl<'a, P> Deref for ImageCow<'a, P> {
    type Target = Image<P>;

    fn deref(&self) -> &Self::Target {
        match self {
            ImageCow::Owned(i) => i,
            ImageCow::Borrowed(i) => i,
        }
    }
}
impl<'a, P> DerefMut for ImageCow<'a, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            ImageCow::Owned(i) => i,
            ImageCow::Borrowed(i) => i,
        }
    }
}

#[allow(unused)]
pub fn from_const_cow<P: Clone>(size: Size, c: P, out: Option<&mut Image<P>>) -> ImageCow<'_, P> {
    if let Some(out) = out {
        assert_eq!(out.size(), size);
        out.fill(c);
        ImageCow::Borrowed(out)
    } else {
        ImageCow::Owned(Image::from_const(size, c))
    }
}

pub fn from_image_cow<'a, P: Copy>(
    img: &Image<P>,
    out: Option<&'a mut Image<P>>,
) -> ImageCow<'a, P> {
    if let Some(out) = out {
        assert_eq!(out.size(), img.size());
        out.data_mut().copy_from_slice(img.data());
        ImageCow::Borrowed(out)
    } else {
        ImageCow::Owned(img.clone())
    }
}
