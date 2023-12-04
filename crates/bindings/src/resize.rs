use glam::{Vec2, Vec3A, Vec4};
use image_core::{ClipFloat, FlattenData, Image, Size};
use image_ops::scale::{CorrectGamma, Filter, GammaCorrection, NoGammaCorrection, ResizePixel};
use numpy::{IntoPyArray, PyArray3, PyReadonlyArrayDyn};
use pyo3::{exceptions::PyValueError, prelude::*};

use crate::{convert::get_channels, load_image, IntoNumpy};

#[pyclass]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResizeFilter {
    Nearest = 0,
    Linear = 1,
    CubicCatrom = 2,
    CubicMitchell = 3,
    CubicBSpline = 6,
    Lanczos = 4,
    Gauss = 5,
}

impl From<ResizeFilter> for Filter {
    fn from(f: ResizeFilter) -> Self {
        match f {
            ResizeFilter::Nearest => Filter::Nearest,
            ResizeFilter::Linear => Filter::Linear,
            ResizeFilter::CubicCatrom => Filter::CubicCatrom,
            ResizeFilter::CubicMitchell => Filter::CubicMitchell,
            ResizeFilter::CubicBSpline => Filter::CubicBSpline,
            ResizeFilter::Lanczos => Filter::Lanczos3,
            ResizeFilter::Gauss => Filter::Gauss,
        }
    }
}

#[pyfunction]
pub fn resize<'py>(
    py: Python<'py>,
    img: PyReadonlyArrayDyn<'py, f32>,
    new_size: (u32, u32),
    filter: ResizeFilter,
    mut gamma_correction: bool,
) -> PyResult<&'py PyArray3<f32>> {
    let new_size = Size::new(new_size.0 as usize, new_size.1 as usize);
    let filter: Filter = filter.into();

    if filter == Filter::Nearest {
        // no point in paying for gamma correction if we're not interpolating
        gamma_correction = false;
    }

    let c = get_channels(&img);
    return match c {
        1 => {
            let img: Image<f32> = load_image!(img);

            if gamma_correction {
                with_pixel_format(py, &img, new_size, filter, GammaCorrection)
            } else {
                with_pixel_format(py, &img, new_size, filter, NoGammaCorrection)
            }
        },
        2 => {
            let img: Image<Vec2> = load_image!(img);

            if gamma_correction {
                with_pixel_format(py, &img, new_size, filter, GammaCorrection)
            } else {
                with_pixel_format(py, &img, new_size, filter, NoGammaCorrection)
            }
        },
        3 => {
            let img: Image<Vec3A> = load_image!(img);

            if gamma_correction {
                with_pixel_format(py, &img, new_size, filter, GammaCorrection)
            } else {
                with_pixel_format(py, &img, new_size, filter, NoGammaCorrection)
            }
        },
        4 => {
            let img: Image<Vec4> = load_image!(img);

            if gamma_correction {
                with_pixel_format(py, &img, new_size, filter, GammaCorrection)
            } else {
                with_pixel_format(py, &img, new_size, filter, NoGammaCorrection)
            }
        },
        _ => Err(PyValueError::new_err(format!(
                "Argument '{}' does not have the right shape. Expected 1, 2, 3, or 4 channels but found {}.",
                stringify!(img),
                c
            ))),
    };

    fn with_pixel_format<P, G, 'py>(
        py: Python<'py>,
        img: &Image<P>,
        new_size: Size,
        filter: Filter,
        gamma_correction: G,
    ) -> PyResult<&'py PyArray3<f32>>
    where
        P: ResizePixel + FlattenData + ClipFloat + 'static,
        G: CorrectGamma<P> + Send + Sync + 'static,
    {
        let result = py.allow_threads(|| {
            let r = image_ops::scale::scale(img, new_size, filter, gamma_correction);
            match r {
                Ok(mut r) => {
                    if filter != Filter::Nearest && filter != Filter::Linear {
                        // the filters may overshoot, so we have to clip the result
                        r.data_mut().iter_mut().for_each(|x| *x = x.clip(0.0, 1.0));
                    }
                    Ok(r.into_numpy())
                }

                Err(_) => Err(PyValueError::new_err(format!(
                    "Not enough memory to allocate a {}x{} image.",
                    new_size.width, new_size.height,
                ))),
            }
        })?;

        Ok(result.into_pyarray(py))
    }
}
