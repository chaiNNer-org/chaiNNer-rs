use crate::{pixel::Flatten, FromFlat, Image, Size};

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

    pub fn data(&self) -> &'a [f32] {
        self.data
    }
}

#[derive(Debug)]
pub enum NDimCow<'a> {
    Image(NDimImage),
    View(NDimView<'a>),
}

impl<'a> NDimCow<'a> {
    pub fn shape(&self) -> Shape {
        match self {
            Self::Image(image) => image.shape(),
            Self::View(view) => view.shape(),
        }
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

    pub fn view(&self) -> NDimView {
        match self {
            Self::Image(image) => image.view(),
            Self::View(view) => *view,
        }
    }
    pub fn into_owned(self) -> NDimImage {
        match self {
            Self::Image(image) => image,
            Self::View(view) => NDimImage::new(view.shape, view.data.to_vec()),
        }
    }

    pub fn data(&self) -> &[f32] {
        match self {
            Self::Image(image) => image.data(),
            Self::View(view) => view.data(),
        }
    }
}

impl From<NDimImage> for NDimCow<'static> {
    fn from(value: NDimImage) -> Self {
        Self::Image(value)
    }
}
impl<'a> From<NDimView<'a>> for NDimCow<'a> {
    fn from(value: NDimView<'a>) -> Self {
        Self::View(value)
    }
}

// Conversions from Image to NDimImage

impl<P: Flatten> From<Image<P>> for NDimImage {
    fn from(value: Image<P>) -> Self {
        Self::new(
            Shape::from_size(value.size(), P::COMPONENTS),
            P::flatten_pixels(value.take()),
        )
    }
}

// Conversions from NDim to Image

#[derive(Debug, PartialEq, Eq)]
pub struct ShapeMismatch {
    pub actual: usize,
    pub expected: Vec<usize>,
}

pub trait IntoPixels<P> {
    fn into_pixels(self) -> Result<Image<P>, ShapeMismatch>;
}
impl<P> IntoPixels<P> for NDimImage
where
    P: FromFlat,
{
    fn into_pixels(self) -> Result<Image<P>, ShapeMismatch> {
        let size = self.size();
        let channels = self.channels();
        match P::from_flat_vec(self.take(), channels) {
            Ok(data) => Ok(Image::new(size, data)),
            Err(e) => Err(ShapeMismatch {
                actual: channels,
                expected: e.supported.to_vec(),
            }),
        }
    }
}
impl<P> IntoPixels<P> for NDimView<'_>
where
    P: FromFlat,
{
    fn into_pixels(self) -> Result<Image<P>, ShapeMismatch> {
        let size = self.size();
        let channels = self.channels();
        match P::from_flat_slice(self.data(), channels) {
            Ok(data) => Ok(Image::new(size, data.into_owned())),
            Err(e) => Err(ShapeMismatch {
                actual: channels,
                expected: e.supported.to_vec(),
            }),
        }
    }
}
