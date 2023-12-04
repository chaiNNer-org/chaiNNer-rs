use std::borrow::Cow;

use glam::{Vec2, Vec3, Vec3A, Vec4};
use image_core::{AsPixels, Image, NDimImage, NDimView, Shape, ShapeMismatch, Size};
use numpy::{ndarray::Array3, IntoPyArray, Ix3, PyArray, PyReadonlyArrayDyn};
use pyo3::Python;

pub fn get_channels(img: &PyReadonlyArrayDyn<f32>) -> usize {
    let data = img.shape();
    if data.len() >= 3 {
        data[2]
    } else {
        1
    }
}

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

impl<T: Into<NDimImage>> IntoNumpy for T {
    fn into_numpy(self) -> Array3<f32> {
        let image: NDimImage = self.into();
        new_numpy_array(image.size(), image.channels(), image.take())
    }
}

pub trait IntoPy {
    fn into_py(self, py: Python<'_>) -> &PyArray<f32, Ix3>;
}
impl<T: IntoNumpy> IntoPy for T {
    fn into_py(self, py: Python<'_>) -> &PyArray<f32, Ix3> {
        self.into_numpy().into_pyarray(py)
    }
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
        NDimView::new(shape, &data).as_pixels()
    }
}
impl<'py, const N: usize> ToOwnedImage<Image<[f32; N]>> for PyReadonlyArrayDyn<'py, f32> {
    fn to_owned_image(&self) -> Result<Image<[f32; N]>, ShapeMismatch> {
        let (shape, data) = read_numpy(self);
        NDimView::new(shape, &data).as_pixels()
    }
}
impl<'py> ToOwnedImage<Image<Vec2>> for PyReadonlyArrayDyn<'py, f32> {
    fn to_owned_image(&self) -> Result<Image<Vec2>, ShapeMismatch> {
        let (shape, data) = read_numpy(self);
        NDimView::new(shape, &data).as_pixels()
    }
}
impl<'py> ToOwnedImage<Image<Vec3>> for PyReadonlyArrayDyn<'py, f32> {
    fn to_owned_image(&self) -> Result<Image<Vec3>, ShapeMismatch> {
        let (shape, data) = read_numpy(self);
        NDimView::new(shape, &data).as_pixels()
    }
}
impl<'py> ToOwnedImage<Image<Vec3A>> for PyReadonlyArrayDyn<'py, f32> {
    fn to_owned_image(&self) -> Result<Image<Vec3A>, ShapeMismatch> {
        let (shape, data) = read_numpy(self);
        NDimView::new(shape, &data).as_pixels()
    }
}
impl<'py> ToOwnedImage<Image<Vec4>> for PyReadonlyArrayDyn<'py, f32> {
    fn to_owned_image(&self) -> Result<Image<Vec4>, ShapeMismatch> {
        let (shape, data) = read_numpy(self);
        NDimView::new(shape, &data).as_pixels()
    }
}
