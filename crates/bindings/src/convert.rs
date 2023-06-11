use std::borrow::Cow;

use glam::{Vec2, Vec3, Vec3A, Vec4};
use image_core::{
    util::{slice_as_chunks, vec_into_flattened},
    Image, NDimImage, Shape, Size,
};
use numpy::{ndarray::Array3, IntoPyArray, Ix3, PyArray, PyReadonlyArrayDyn};
use pyo3::Python;

fn py_to_shape(shape: &[usize]) -> Shape {
    if shape.len() == 2 {
        Shape::new(shape[1], shape[0], 1)
    } else {
        assert_eq!(shape.len(), 3);
        // python shape is in height width channels
        Shape::new(shape[1], shape[0], shape[2])
    }
}

fn new_numpy_array(size: Size, channels: usize, data: Vec<f32>) -> Array3<f32> {
    let shape = Ix3(size.height, size.width, channels);
    Array3::from_shape_vec(shape, data).expect("Expect creation of numpy array to succeed.")
}

fn read_numpy<'a>(ndarray: &'a PyReadonlyArrayDyn<f32>) -> (Shape, Cow<'a, [f32]>) {
    let shape = py_to_shape(ndarray.shape());

    if ndarray.is_c_contiguous() {
        if let Ok(data) = ndarray.as_slice() {
            return (shape, Cow::Borrowed(data));
        }
    }

    let data = ndarray.as_array().iter().cloned().collect();
    (shape, Cow::Owned(data))
}

pub trait IntoNumpy {
    fn into_numpy(self) -> Array3<f32>;
}

impl IntoNumpy for NDimImage {
    fn into_numpy(self) -> Array3<f32> {
        new_numpy_array(self.size(), self.channels(), self.take())
    }
}
impl IntoNumpy for Image<f32> {
    fn into_numpy(self) -> Array3<f32> {
        new_numpy_array(self.size(), 1, self.take())
    }
}
impl<const N: usize> IntoNumpy for Image<[f32; N]> {
    fn into_numpy(self) -> Array3<f32> {
        new_numpy_array(self.size(), N, vec_into_flattened(self.take()))
    }
}

macro_rules! generate_into_numpy_fn {
    ($for_type:ty, $n:literal, $f:expr) => {
        impl IntoNumpy for Image<$for_type> {
            fn into_numpy(self) -> Array3<f32> {
                let size = self.size();
                let data: Vec<[f32; $n]> = self.take().into_iter().map($f).collect();
                new_numpy_array(size, $n, vec_into_flattened(data))
            }
        }
    };
}
macro_rules! generate_into_numpy_array {
    ($for_type:ty, $n:literal) => {
        generate_into_numpy_fn!($for_type, $n, |v| v.into());
    };
}
generate_into_numpy_array!(Vec4, 4);
generate_into_numpy_array!(Vec3, 3);
generate_into_numpy_array!(Vec3A, 3);
generate_into_numpy_array!(Vec2, 2);
generate_into_numpy_fn!((f32, f32), 2, |(x, y)| [x, y]);

pub trait IntoPy {
    fn into_py(self, py: Python<'_>) -> &PyArray<f32, Ix3>;
}
impl<T: IntoNumpy> IntoPy for T {
    fn into_py(self, py: Python<'_>) -> &PyArray<f32, Ix3> {
        self.into_numpy().into_pyarray(py)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ShapeMismatch {
    pub actual: usize,
    pub expected: Vec<usize>,
}

pub trait ToOwnedImage<T> {
    fn to_owned_image(&self) -> Result<T, ShapeMismatch>;
}

impl<'py> ToOwnedImage<NDimImage> for PyReadonlyArrayDyn<'py, f32> {
    fn to_owned_image(&self) -> Result<NDimImage, ShapeMismatch> {
        let (shape, data) = read_numpy(self);
        let data = match data {
            Cow::Borrowed(s) => s.to_vec(),
            Cow::Owned(v) => v,
        };
        Ok(NDimImage::new(shape, data))
    }
}
impl<'py> ToOwnedImage<Image<f32>> for PyReadonlyArrayDyn<'py, f32> {
    fn to_owned_image(&self) -> Result<Image<f32>, ShapeMismatch> {
        let (shape, data) = read_numpy(self);
        if shape.channels == 1 {
            let data = match data {
                Cow::Borrowed(s) => s.to_vec(),
                Cow::Owned(v) => v,
            };
            return Ok(Image::new(shape.size(), data));
        }

        Err(ShapeMismatch {
            actual: shape.channels,
            expected: vec![1],
        })
    }
}
impl<'py, const N: usize> ToOwnedImage<Image<[f32; N]>> for PyReadonlyArrayDyn<'py, f32> {
    fn to_owned_image(&self) -> Result<Image<[f32; N]>, ShapeMismatch> {
        let (shape, data) = read_numpy(self);

        if shape.channels == N {
            let (chunks, rest) = slice_as_chunks(&data);
            assert!(rest.is_empty());
            return Ok(Image::new(shape.size(), chunks.to_vec()));
        }

        Err(ShapeMismatch {
            actual: shape.channels,
            expected: vec![N],
        })
    }
}
impl<'py> ToOwnedImage<Image<Vec4>> for PyReadonlyArrayDyn<'py, f32> {
    fn to_owned_image(&self) -> Result<Image<Vec4>, ShapeMismatch> {
        let (shape, data) = read_numpy(self);

        if shape.channels == 1 {
            let data = data.iter().map(|g| Vec4::new(*g, *g, *g, 1.)).collect();
            return Ok(Image::new(shape.size(), data));
        }
        if shape.channels == 3 {
            let (chunks, rest) = slice_as_chunks::<_, 3>(&data);
            assert!(rest.is_empty());
            let data = chunks
                .iter()
                .map(|[x, y, z]| Vec4::new(*x, *y, *z, 1.))
                .collect();
            return Ok(Image::new(shape.size(), data));
        }
        if shape.channels == 4 {
            let (chunks, rest) = slice_as_chunks::<_, 4>(&data);
            assert!(rest.is_empty());
            let data = chunks
                .iter()
                .map(|[x, y, z, w]| Vec4::new(*x, *y, *z, *w))
                .collect();
            return Ok(Image::new(shape.size(), data));
        }

        Err(ShapeMismatch {
            actual: shape.channels,
            expected: vec![1, 3, 4],
        })
    }
}
