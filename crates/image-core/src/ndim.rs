use crate::{pixel::FlattenData, util::slice_as_chunks, Image, Size};
use glam::{Vec2, Vec3, Vec3A, Vec4};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Shape {
    pub width: usize,
    pub height: usize,
    pub channels: usize,
}

impl Shape {
    pub fn new(width: usize, height: usize, channels: usize) -> Self {
        Self {
            width,
            height,
            channels,
        }
    }

    pub fn from_size(size: Size, channels: usize) -> Self {
        Self {
            width: size.width,
            height: size.height,
            channels,
        }
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
    pub fn len(&self) -> usize {
        self.width * self.height * self.channels
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A 3D image that is similar to how numpy arrays.
#[derive(Debug, Clone)]
pub struct NDimImage {
    data: Vec<f32>,
    shape: Shape,
}

impl NDimImage {
    pub fn new(shape: Shape, data: Vec<f32>) -> Self {
        assert!(shape.len() == data.len());
        Self { data, shape }
    }
    pub fn zeros(shape: Shape) -> Self {
        Self::new(shape, vec![0f32; shape.len()])
    }
    pub fn from_fn_c<const C: usize>(size: Size, f: impl Fn(usize, usize) -> [f32; C]) -> Self {
        let shape = Shape::from_size(size, C);
        let f = &f;

        Self::new(
            shape,
            (0..shape.height)
                .flat_map(|y| (0..shape.width).flat_map(move |x| f(x, y)))
                .collect(),
        )
    }

    pub fn take(self) -> Vec<f32> {
        self.data
    }
    pub fn view(&self) -> NDimView {
        NDimView::new(self.shape, &self.data)
    }

    pub fn shape(&self) -> Shape {
        self.shape
    }
    pub fn size(&self) -> Size {
        self.shape().size()
    }
    pub fn width(&self) -> usize {
        self.shape().width
    }
    pub fn height(&self) -> usize {
        self.shape().height
    }
    pub fn channels(&self) -> usize {
        self.shape().channels
    }

    pub fn data(&self) -> &[f32] {
        &self.data[..]
    }
    pub fn data_mut(&mut self) -> &mut [f32] {
        &mut self.data[..]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NDimView<'a> {
    data: &'a [f32],
    shape: Shape,
}

impl<'a> NDimView<'a> {
    pub fn new(shape: Shape, data: &'a [f32]) -> Self {
        assert!(shape.len() == data.len());
        Self { data, shape }
    }

    pub fn shape(&self) -> Shape {
        self.shape
    }
    pub fn size(&self) -> Size {
        self.shape().size()
    }
    pub fn width(&self) -> usize {
        self.shape().width
    }
    pub fn height(&self) -> usize {
        self.shape().height
    }
    pub fn channels(&self) -> usize {
        self.shape().channels
    }

    pub fn data(&self) -> &[f32] {
        self.data
    }
}

// Conversions from Image to NDimImage

impl<P: FlattenData> From<Image<P>> for NDimImage {
    fn from(value: Image<P>) -> Self {
        Self::new(
            Shape::from_size(value.size(), P::COMPONENTS),
            P::flatten_data(value.take()),
        )
    }
}

// Conversions from NDim to Image

#[derive(Debug, PartialEq, Eq)]
pub struct ShapeMismatch {
    pub actual: usize,
    pub expected: Vec<usize>,
}

pub trait AsPixels<P> {
    fn as_pixels(&self) -> Result<Image<P>, ShapeMismatch>;
}

impl<P> AsPixels<P> for NDimImage
where
    for<'a> NDimView<'a>: AsPixels<P>,
{
    fn as_pixels(&self) -> Result<Image<P>, ShapeMismatch> {
        self.view().as_pixels()
    }
}

impl<'a> AsPixels<f32> for NDimView<'a> {
    fn as_pixels(&self) -> Result<Image<f32>, ShapeMismatch> {
        if self.channels() == 1 {
            return Ok(Image::new(self.size(), self.data.to_vec()));
        }

        Err(ShapeMismatch {
            actual: self.channels(),
            expected: vec![1],
        })
    }
}
impl<'a, const N: usize> AsPixels<[f32; N]> for NDimView<'a> {
    fn as_pixels(&self) -> Result<Image<[f32; N]>, ShapeMismatch> {
        if self.channels() == N {
            let (chunks, rest) = slice_as_chunks(self.data);
            assert!(rest.is_empty());
            return Ok(Image::new(self.size(), chunks.to_vec()));
        }

        Err(ShapeMismatch {
            actual: self.channels(),
            expected: vec![N],
        })
    }
}
impl<'a> AsPixels<Vec2> for NDimView<'a> {
    fn as_pixels(&self) -> Result<Image<Vec2>, ShapeMismatch> {
        if self.channels() == 1 {
            let data = self.data.iter().map(|g| Vec2::new(*g, *g)).collect();
            return Ok(Image::new(self.size(), data));
        }
        if self.channels() == 2 {
            let (chunks, rest) = slice_as_chunks::<_, 2>(self.data);
            assert!(rest.is_empty());
            let data = chunks.iter().map(|[x, y]| Vec2::new(*x, *y)).collect();
            return Ok(Image::new(self.size(), data));
        }

        Err(ShapeMismatch {
            actual: self.channels(),
            expected: vec![1, 2],
        })
    }
}
impl<'a> AsPixels<Vec3> for NDimView<'a> {
    fn as_pixels(&self) -> Result<Image<Vec3>, ShapeMismatch> {
        if self.channels() == 1 {
            let data = self.data.iter().map(|g| Vec3::new(*g, *g, *g)).collect();
            return Ok(Image::new(self.size(), data));
        }
        if self.channels() == 3 {
            let (chunks, rest) = slice_as_chunks::<_, 3>(self.data);
            assert!(rest.is_empty());
            let data = chunks
                .iter()
                .map(|[x, y, z]| Vec3::new(*x, *y, *z))
                .collect();
            return Ok(Image::new(self.size(), data));
        }

        Err(ShapeMismatch {
            actual: self.channels(),
            expected: vec![1, 3],
        })
    }
}
impl<'a> AsPixels<Vec3A> for NDimView<'a> {
    fn as_pixels(&self) -> Result<Image<Vec3A>, ShapeMismatch> {
        if self.channels() == 1 {
            let data = self.data.iter().map(|g| Vec3A::new(*g, *g, *g)).collect();
            return Ok(Image::new(self.size(), data));
        }
        if self.channels() == 3 {
            let (chunks, rest) = slice_as_chunks::<_, 3>(self.data);
            assert!(rest.is_empty());
            let data = chunks
                .iter()
                .map(|[x, y, z]| Vec3A::new(*x, *y, *z))
                .collect();
            return Ok(Image::new(self.size(), data));
        }

        Err(ShapeMismatch {
            actual: self.channels(),
            expected: vec![1, 3],
        })
    }
}
impl<'a> AsPixels<Vec4> for NDimView<'a> {
    fn as_pixels(&self) -> Result<Image<Vec4>, ShapeMismatch> {
        if self.channels() == 1 {
            let data = self
                .data
                .iter()
                .map(|g| Vec4::new(*g, *g, *g, 1.))
                .collect();
            return Ok(Image::new(self.size(), data));
        }
        if self.channels() == 3 {
            let (chunks, rest) = slice_as_chunks::<_, 3>(self.data);
            assert!(rest.is_empty());
            let data = chunks
                .iter()
                .map(|[x, y, z]| Vec4::new(*x, *y, *z, 1.))
                .collect();
            return Ok(Image::new(self.size(), data));
        }
        if self.channels() == 4 {
            let (chunks, rest) = slice_as_chunks::<_, 4>(self.data);
            assert!(rest.is_empty());
            let data = chunks
                .iter()
                .map(|[x, y, z, w]| Vec4::new(*x, *y, *z, *w))
                .collect();
            return Ok(Image::new(self.size(), data));
        }

        Err(ShapeMismatch {
            actual: self.channels(),
            expected: vec![1, 3, 4],
        })
    }
}
