use glam::{Vec2, Vec3A, Vec4};
use image_core::{ClipFloat, Flatten, FromFlat, Image, ImageView, IntoPixels, NDimImage, Size};
use image_ops::scale::{Filter, FloatPixelFormat, PixelFormat};
use numpy::{IntoPyArray, PyArray3};
use pyo3::{exceptions::PyValueError, prelude::*};

use crate::{
    convert::{LoadImage, PyImage, ViewImage},
    IntoNumpy,
};

#[pyclass]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResizeFilter {
    Nearest = 0,
    Box = 8,
    Linear = 1,
    Hermite = 9,
    CubicCatrom = 2,
    CubicMitchell = 3,
    CubicBSpline = 6,
    Hamming = 10,
    Hann = 11,
    Lanczos = 4,
    Lagrange = 7,
    Gauss = 5,
}

impl From<ResizeFilter> for Filter {
    fn from(f: ResizeFilter) -> Self {
        match f {
            ResizeFilter::Nearest => Filter::Nearest,
            ResizeFilter::Box => Filter::Box,
            ResizeFilter::Linear => Filter::Linear,
            ResizeFilter::Hermite => Filter::Hermite,
            ResizeFilter::CubicCatrom => Filter::CubicCatrom,
            ResizeFilter::CubicMitchell => Filter::CubicMitchell,
            ResizeFilter::CubicBSpline => Filter::CubicBSpline,
            ResizeFilter::Hamming => Filter::Hamming,
            ResizeFilter::Hann => Filter::Hann,
            ResizeFilter::Lanczos => Filter::Lanczos3,
            ResizeFilter::Lagrange => Filter::Lagrange,
            ResizeFilter::Gauss => Filter::Gauss,
        }
    }
}

#[pyfunction]
pub fn resize<'py>(
    py: Python<'py>,
    img: PyImage<'py>,
    new_size: (u32, u32),
    filter: ResizeFilter,
    mut gamma_correction: bool,
) -> PyResult<&'py PyArray3<f32>> {
    let new_size: Size = new_size.into();
    let filter: Filter = filter.into();

    if filter == Filter::Nearest {
        // no point in paying for gamma correction if we're not interpolating
        gamma_correction = false;
    }

    let c = img.channels();

    let new_error = || {
        PyValueError::new_err(format!(
            "Argument '{}' does not have the right shape. Expected 1, 2, 3, or 4 channels but found {}.",
            stringify!(img),
            c
        ))
    };

    if gamma_correction {
        let mut img: NDimImage = img.load_image()?;
        let result: PyResult<_> = py.allow_threads(|| {
            // convert to linear
            image_ops::gamma::gamma_ndim(&mut img, 2.2);

            // the actual resizing
            let mut result = match c {
                1 => with_pixel_format::<f32>(img, new_size, filter)?,
                2 => with_pixel_format::<Vec2>(img, new_size, filter)?,
                3 => with_pixel_format::<Vec3A>(img, new_size, filter)?,
                4 => with_pixel_format::<Vec4>(img, new_size, filter)?,
                _ => return Err(new_error()),
            };

            // fix up overshooting
            if filter != Filter::Nearest && filter != Filter::Linear {
                // the filters may overshoot, so we have to clip the result
                result
                    .data_mut()
                    .iter_mut()
                    .for_each(|x| *x = x.clip(0.0, 1.0));
            }

            // convert back to sRGB
            image_ops::gamma::gamma_ndim(&mut result, 1.0 / 2.2);

            return Ok(result.into_numpy());

            fn with_pixel_format<P>(
                img: NDimImage,
                new_size: Size,
                filter: Filter,
            ) -> PyResult<NDimImage>
            where
                P: Flatten + FromFlat + Default + Clone + 'static,
                FloatPixelFormat<P>: PixelFormat<InputPixel = P, OutputPixel = P>,
            {
                let img: Image<P> = img.into_pixels().expect("");
                let r = image_ops::scale::scale(img.view(), new_size, filter);

                // drop image now to free up memory asap
                std::mem::drop(img);

                match r {
                    Ok(r) => Ok(r.into()),
                    Err(_) => Err(PyValueError::new_err(format!(
                        "Not enough memory to allocate a {}x{} image.",
                        new_size.width, new_size.height,
                    ))),
                }
            }
        });

        return Ok(result?.into_pyarray(py));
    }

    {
        // read the image directly if we can to avoid copying

        if let Some(view) = img.view_image() {
            return with_pixel_format::<f32>(py, view, new_size, filter);
        }
        if let Some(view) = img.view_image() {
            return with_pixel_format::<[f32; 3]>(py, view, new_size, filter);
        }
        if let Some(view) = img.view_image() {
            return with_pixel_format::<[f32; 4]>(py, view, new_size, filter);
        }

        fn with_pixel_format<'py, P>(
            py: Python<'py>,
            img: ImageView<'_, P>,
            new_size: Size,
            filter: Filter,
        ) -> PyResult<&'py PyArray3<f32>>
        where
            P: Flatten + ClipFloat + Default + Copy + Sync + Send + 'static,
            FloatPixelFormat<P>: PixelFormat<InputPixel = P, OutputPixel = P>,
        {
            let r = image_ops::scale::scale(img, new_size, filter);
            match r {
                Ok(mut r) => {
                    if filter != Filter::Nearest && filter != Filter::Linear {
                        // the filters may overshoot, so we have to clip the result
                        r.data_mut().iter_mut().for_each(|x| *x = x.clip(0.0, 1.0));
                    }
                    Ok(r.into_numpy().into_pyarray(py))
                }

                Err(_) => Err(PyValueError::new_err(format!(
                    "Not enough memory to allocate a {}x{} image.",
                    new_size.width, new_size.height,
                ))),
            }
        }
    }

    // Using vector types involves at least one copy to convert `Vec<[f32; N]>`
    // -> `Vec<VecN>`. This is a significant overhead that isn't worth it if we
    // don't upscale by a large amount (=a lot of computation).
    let mut vec_worth = false;

    let src_pixels = img.size().len();
    let dst_pixels = new_size.len();
    let scale_factor = dst_pixels as f64 / src_pixels as f64;

    if c == 4 && scale_factor >= 1.99 {
        vec_worth = true;
    }
    if c == 3 {
        // Using Vec3A is never worth it. The issue is that we have to pay for
        // the conversion [f32; 3] <-> Vec3A both ways, which makes it really
        // expensive. For some slower filters, it actually is little bit worth
        // it (scales>3 are ~10% faster), but we want to avoid the additional
        // memory usage.
        vec_worth = false;
    }
    if filter == Filter::Nearest {
        // NN doesn't interpolate pixels
        vec_worth = false;
    }

    return match c {
        1 => {
            let img: Image<f32> = img.load_image()?;
            with_pixel_format(py, img, new_size, filter)
        }
        2 => {
            let img: Image<[f32; 2]> = img.load_image()?;
            with_pixel_format(py, img, new_size, filter)
        }
        3 => {
            let img: Image<[f32; 3]> = img.load_image()?;
            with_pixel_format(py, img, new_size, filter)
        }
        4 => {
            if vec_worth {
                let img: Image<Vec4> = img.load_image()?;
                with_pixel_format(py, img, new_size, filter)
            } else {
                let img: Image<[f32; 4]> = img.load_image()?;
                with_pixel_format(py, img, new_size, filter)
            }
        }
        _ => Err(new_error()),
    };

    fn with_pixel_format<P>(
        py: Python,
        img: Image<P>,
        new_size: Size,
        filter: Filter,
    ) -> PyResult<&PyArray3<f32>>
    where
        P: Flatten + ClipFloat + Default + Copy + Send + 'static,
        FloatPixelFormat<P>: PixelFormat<InputPixel = P, OutputPixel = P>,
    {
        let result = py.allow_threads(|| {
            let r = image_ops::scale::scale(img.view(), new_size, filter);
            std::mem::drop(img);
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
