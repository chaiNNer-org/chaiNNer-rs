#![feature(slice_flatten, slice_as_chunks, iter_intersperse)]

mod convert;

use glam::Vec4;
use image_core::{Image, Size};
use image_ops::scale::nearest_neighbor;
use numpy::{IntoPyArray, PyArray3, PyReadonlyArrayDyn};
use pyo3::{exceptions::PyValueError, prelude::*};

use crate::convert::{FromNumpy, IntoNumpy, IntoPy};

/// A macro for converting native numpy arrays to images.
///
/// The image type will automatically be inferred based on usage.
/// If the numpy array cannot be converted, an error will be early-returned.
macro_rules! load_image {
    ($img:ident) => {
        match $img.from_numpy() {
            Ok(r) => r,
            Err(e) => {
                return Err(PyValueError::new_err(format!(
                    "Argument '{}' does not have the right shape. Expected {} channel(s) but found {}.",
                    stringify!($img),
                    e.expected
                        .iter()
                        .map(|s| s.to_string())
                        .intersperse(", ".to_owned())
                        .collect::<String>(),
                    e.actual
                )))
            }
        }
    };
}

/// A Python module implemented in Rust.
#[pymodule]
fn chainner_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    /// Formats the sum of two numbers as string.
    #[pyfn(m)]
    fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
        Ok((a + b).to_string())
    }

    /// Inverts the colors of a given image.
    #[pyfn(m)]
    fn invert<'py>(py: Python<'py>, img: PyReadonlyArrayDyn<f32>) -> PyResult<&'py PyArray3<f32>> {
        let foo = load_image!(img);
        // let img = img.from_numpy().unwrap();
        let result = py.allow_threads(move || nearest_neighbor::<Vec4>(&foo, foo.size().scale(4.)));
        let result = result.into_numpy().into_pyarray(py);
        Ok(result)
    }

    /// Inverts the colors of a given image.
    #[pyfn(m)]
    fn rainbow(py: Python<'_>) -> PyResult<&PyArray3<f32>> {
        let result = py.allow_threads(move || {
            let rainbow = Image::from_fn(Size::new(256, 256), |x, y| {
                [x as f32 / 255., y as f32 / 255., 0.]
            });
            rainbow
        });
        let result = result.into_py(py);
        Ok(result)
    }

    Ok(())
}
