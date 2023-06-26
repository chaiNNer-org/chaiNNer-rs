mod convert;
mod regex;

use glam::Vec4;
use image_core::{Image, Size};
use image_ops::{
    fill_alpha::{fill_alpha, FillMode},
    scale::nearest_neighbor,
};
use numpy::{IntoPyArray, PyArray3, PyReadonlyArrayDyn};
use pyo3::{exceptions::PyValueError, prelude::*};

use crate::convert::{IntoNumpy, IntoPy, ToOwnedImage};

/// A macro for converting native numpy arrays to images.
///
/// The image type will automatically be inferred based on usage.
/// If the numpy array cannot be converted, an error will be early-returned.
macro_rules! load_image {
    ($img:ident) => {
        match $img.to_owned_image() {
            Ok(r) => r,
            Err(e) => {
                return Err(PyValueError::new_err(format!(
                    "Argument '{}' does not have the right shape. Expected {} channel(s) but found {}.",
                    stringify!($img),
                    e.expected
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    e.actual
                )))
            }
        }
    };
}

/// A Python module implemented in Rust.
#[pymodule]
fn chainner_ext(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<regex::RustRegex>()?;
    m.add_class::<regex::MatchGroup>()?;
    m.add_class::<regex::RegexMatch>()?;

    /// Test function
    #[pyfn(m)]
    fn test_invert<'py>(
        py: Python<'py>,
        img: PyReadonlyArrayDyn<f32>,
    ) -> PyResult<&'py PyArray3<f32>> {
        let img = load_image!(img);
        // let img = img.from_numpy().unwrap();
        let result = py.allow_threads(move || nearest_neighbor::<Vec4>(&img, img.size().scale(4.)));
        let result = result.into_numpy().into_pyarray(py);
        Ok(result)
    }

    /// Test function
    #[pyfn(m)]
    fn test_rainbow(py: Python<'_>) -> PyResult<&PyArray3<f32>> {
        let result = py.allow_threads(move || {
            Image::from_fn(Size::new(256, 256), |x, y| {
                [x as f32 / 255., y as f32 / 255., 0.]
            })
        });
        let result = result.into_py(py);
        Ok(result)
    }

    /// Fill the transparent pixels in the given image with nearby colors.
    #[pyfn(m)]
    fn fill_alpha_fragment_blur<'py>(
        py: Python<'py>,
        img: PyReadonlyArrayDyn<f32>,
        threshold: f32,
        iterations: u32,
        fragment_count: u32,
    ) -> PyResult<&'py PyArray3<f32>> {
        let mut img = load_image!(img);
        let result = py.allow_threads(|| {
            fill_alpha(
                &mut img,
                threshold,
                FillMode::Fragment {
                    iterations,
                    fragment_count,
                },
                None,
            );
            img.into_numpy()
        });
        Ok(result.into_pyarray(py))
    }

    /// Fill the transparent pixels in the given image with nearby colors.
    #[pyfn(m)]
    fn fill_alpha_extend_color<'py>(
        py: Python<'py>,
        img: PyReadonlyArrayDyn<f32>,
        threshold: f32,
        iterations: u32,
    ) -> PyResult<&'py PyArray3<f32>> {
        let mut img = load_image!(img);
        let result = py.allow_threads(|| {
            fill_alpha(
                &mut img,
                threshold,
                FillMode::ExtendColor { iterations },
                None,
            );
            img.into_numpy()
        });
        Ok(result.into_pyarray(py))
    }

    /// Fill the transparent pixels in the given image with nearby colors.
    #[pyfn(m)]
    fn fill_alpha_nearest_color<'py>(
        py: Python<'py>,
        img: PyReadonlyArrayDyn<f32>,
        threshold: f32,
        min_radius: u32,
        anti_aliasing: bool,
    ) -> PyResult<&'py PyArray3<f32>> {
        let mut img = load_image!(img);
        let result = py.allow_threads(|| {
            fill_alpha(
                &mut img,
                threshold,
                FillMode::Nearest {
                    min_radius,
                    anti_aliasing,
                },
                None,
            );
            img.into_numpy()
        });
        Ok(result.into_pyarray(py))
    }

    Ok(())
}
