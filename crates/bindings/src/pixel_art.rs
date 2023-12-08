use std::ops::{Add, Mul};

use glam::{Vec3A, Vec4};
use image_core::{FromFlat, Image};
use image_ops::pixel_art::IntoYuv;
use numpy::{IntoPyArray, PyArray3};
use pyo3::{exceptions::PyValueError, prelude::*};

use crate::convert::{IntoNumpy, LoadImage, PyImage};

#[pyfunction]
pub fn pixel_art_upscale<'py>(
    py: Python<'py>,
    img: PyImage<'py>,
    algorithm: &str,
    scale: u32,
) -> PyResult<&'py PyArray3<f32>> {
    fn with_pixel_format<'py, P>(
        py: Python<'py>,
        img: PyImage<'py>,
        algorithm: &str,
        scale: u32,
    ) -> PyResult<&'py PyArray3<f32>>
    where
        P: FromFlat
            + Default
            + Copy
            + PartialEq
            + IntoYuv
            + Add<P, Output = P>
            + Mul<f32, Output = P>
            + Sync,
        Image<P>: IntoNumpy,
    {
        let img: Image<P> = img.load_image()?;
        let result = py.allow_threads(|| {
            let result: Image<P> = match algorithm {
                "adv_mame" => match scale {
                    2 => image_ops::pixel_art::adv_mame_2x(&img),
                    3 => image_ops::pixel_art::adv_mame_3x(&img),
                    4 => image_ops::pixel_art::adv_mame_4x(&img),
                    _ => {
                        return Err(PyValueError::new_err(format!(
                            "Scale {} is not supported for pixel art upscaling algorithm '{}'.",
                            scale, algorithm,
                        )))
                    }
                },
                "eagle" => match scale {
                    2 => image_ops::pixel_art::eagle_2x(&img),
                    3 => image_ops::pixel_art::eagle_3x(&img),
                    _ => {
                        return Err(PyValueError::new_err(format!(
                            "Scale {} is not supported for pixel art upscaling algorithm '{}'.",
                            scale, algorithm,
                        )))
                    }
                },
                "super_eagle" => match scale {
                    2 => image_ops::pixel_art::super_eagle_2x(&img),
                    _ => {
                        return Err(PyValueError::new_err(format!(
                            "Scale {} is not supported for pixel art upscaling algorithm '{}'.",
                            scale, algorithm,
                        )))
                    }
                },
                "sai" => match scale {
                    2 => image_ops::pixel_art::sai_2x(&img),
                    _ => {
                        return Err(PyValueError::new_err(format!(
                            "Scale {} is not supported for pixel art upscaling algorithm '{}'.",
                            scale, algorithm,
                        )))
                    }
                },
                "super_sai" => match scale {
                    2 => image_ops::pixel_art::super_sai_2x(&img),
                    _ => {
                        return Err(PyValueError::new_err(format!(
                            "Scale {} is not supported for pixel art upscaling algorithm '{}'.",
                            scale, algorithm,
                        )))
                    }
                },
                "hqx" => match scale {
                    2 => image_ops::pixel_art::hq2x(&img),
                    3 => image_ops::pixel_art::hq3x(&img),
                    4 => image_ops::pixel_art::hq4x(&img),
                    _ => {
                        return Err(PyValueError::new_err(format!(
                            "Scale {} is not supported for pixel art upscaling algorithm '{}'.",
                            scale, algorithm,
                        )))
                    }
                },
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Unknown pixel art upscaling algorithm '{}'.",
                        algorithm,
                    )))
                }
            };
            Ok(result.into_numpy())
        })?;
        Ok(result.into_pyarray(py))
    }

    let c = img.channels();
    match c {
        1 => with_pixel_format::<f32>(py, img, algorithm,scale),
        3 => with_pixel_format::<Vec3A>(py, img, algorithm,scale),
        4 => with_pixel_format::<Vec4>(py, img, algorithm,scale),
        _ => Err(PyValueError::new_err(format!(
            "Argument '{}' does not have the right shape. Expected 1, 3, or 4 channels but found {}.",
            stringify!(img),
            c
        ))),
    }
}
