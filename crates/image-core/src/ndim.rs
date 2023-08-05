use crate::{util::vec_into_flattened, Image, Size};
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

impl From<Image<f32>> for NDimImage {
    fn from(value: Image<f32>) -> Self {
        Self::new(Shape::from_size(value.size(), 1), value.take())
    }
}
impl<const N: usize> From<Image<[f32; N]>> for NDimImage {
    fn from(value: Image<[f32; N]>) -> Self {
        Self::new(
            Shape::from_size(value.size(), N),
            vec_into_flattened(value.take()),
        )
    }
}

macro_rules! generate_into_ndim_fn {
    ($for_type:ty, $n:literal, $f:expr) => {
        impl From<Image<$for_type>> for NDimImage {
            fn from(value: Image<$for_type>) -> Self {
                let size = value.size();
                let data: Vec<[f32; $n]> = value.take().into_iter().map($f).collect();
                Self::new(Shape::from_size(size, $n), vec_into_flattened(data))
            }
        }
    };
}
macro_rules! generate_into_ndim_array {
    ($for_type:ty, $n:literal) => {
        generate_into_ndim_fn!($for_type, $n, |v| v.into());
    };
}
generate_into_ndim_array!(Vec4, 4);
generate_into_ndim_array!(Vec3, 3);
generate_into_ndim_array!(Vec3A, 3);
generate_into_ndim_array!(Vec2, 2);
generate_into_ndim_fn!((f32, f32), 2, |(x, y)| [x, y]);
