mod clipboard;
mod convert;
mod dither;
mod macros;
mod regex;

use image_core::{Image, NDimImage};
use image_ops::fill_alpha::{fill_alpha, FillMode};
use numpy::{IntoPyArray, PyArray3, PyReadonlyArrayDyn};
use pyo3::{exceptions::PyValueError, prelude::*};

use crate::convert::IntoNumpy;

/// A Python module implemented in Rust.
#[pymodule]
fn chainner_ext(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<regex::RustRegex>()?;
    m.add_class::<regex::MatchGroup>()?;
    m.add_class::<regex::RegexMatch>()?;

    m.add_class::<clipboard::Clipboard>()?;

    m.add_class::<dither::DiffusionAlgorithm>()?;
    m.add_class::<dither::UniformQuantization>()?;
    m.add_class::<dither::PaletteQuantization>()?;
    m.add_wrapped(wrap_pyfunction!(dither::quantize))?;
    m.add_wrapped(wrap_pyfunction!(dither::error_diffusion_dither))?;
    m.add_wrapped(wrap_pyfunction!(dither::ordered_dither))?;
    m.add_wrapped(wrap_pyfunction!(dither::riemersma_dither))?;

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

    /// Fill the transparent pixels in the given image with nearby colors.
    #[pyfn(m)]
    fn binary_threshold<'py>(
        py: Python<'py>,
        img: PyReadonlyArrayDyn<f32>,
        threshold: f32,
        anti_aliasing: bool,
    ) -> PyResult<&'py PyArray3<f32>> {
        let img: NDimImage = load_image!(img);
        let result = py.allow_threads(|| {
            image_ops::threshold::binary_threshold(img.view(), threshold, anti_aliasing)
                .into_numpy()
        });
        Ok(result.into_pyarray(py))
    }

    /// Fill the transparent pixels in the given image with nearby colors.
    #[pyfn(m)]
    fn esdf<'py>(
        py: Python<'py>,
        img: PyReadonlyArrayDyn<f32>,
        radius: f32,
        cutoff: f32,
        pre_process: bool,
        post_process: bool,
    ) -> PyResult<&'py PyArray3<f32>> {
        let img: Image<f32> = load_image!(img);
        let result = py.allow_threads(|| {
            image_ops::esdt::esdf(&img, radius, cutoff, pre_process, post_process).into_numpy()
        });
        Ok(result.into_pyarray(py))
    }

    Ok(())
}
