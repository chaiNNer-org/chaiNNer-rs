use std::sync::Arc;

use glam::{Vec3A, Vec4};
use image_core::{FromFlat, Image, IntoPixels, NDimImage};
use image_ops::{
    dither::*,
    palette::{extract_unique_ndim, ExtractionError},
};
use numpy::{IntoPyArray, PyArray3};
use pyo3::{exceptions::PyValueError, prelude::*};

use crate::convert::{IntoNumpy, LoadImage, PyImage};

#[pyclass(frozen)]
#[derive(Clone, PartialEq, Debug)]
pub struct UniformQuantization {
    inner: ChannelQuantization,
}

#[pymethods]
impl UniformQuantization {
    #[new]
    pub fn new(colors_per_channel: u32) -> PyResult<Self> {
        if colors_per_channel < 2 {
            return Err(PyValueError::new_err(format!(
                "Argument '{}' must be at least 2.",
                stringify!(per_channel)
            )));
        }

        Ok(Self {
            inner: ChannelQuantization::new(colors_per_channel as usize),
        })
    }

    #[getter]
    pub fn colors_per_channel(&self) -> u32 {
        self.inner.per_channel() as u32
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct PaletteQuantization {
    palette: Arc<NDimImage>,
}

#[pymethods]
impl PaletteQuantization {
    #[new]
    pub fn new(palette: PyImage) -> PyResult<Self> {
        let palette: NDimImage = palette.load_image()?;
        if palette.height() != 1 {
            return Err(PyValueError::new_err(format!(
                "Argument '{}' must have a height of 1.",
                stringify!(palette)
            )));
        }

        let palette = match extract_unique_ndim(palette.view(), usize::MAX) {
            Ok(palette) => palette,
            Err(err) => match err {
                ExtractionError::TooManyColors {
                    max_colors,
                    actual_colors,
                } => {
                    return Err(PyValueError::new_err(format!(
                        "Argument '{}' has too many colors. Expected at most {}, got {}.",
                        stringify!(palette),
                        max_colors,
                        actual_colors
                    )));
                }
                ExtractionError::UnsupportedChannels { channels } => {
                    return Err(PyValueError::new_err(format!(
                        "Argument '{}' has an unsupported number of channels. Images with {} channels are not supported.",
                        stringify!(palette),
                        channels
                    )));
                }
            },
        };

        Ok(Self {
            palette: Arc::new(palette),
        })
    }

    #[getter]
    pub fn channels(&self) -> u32 {
        self.palette.channels() as u32
    }

    #[getter]
    pub fn colors(&self) -> u32 {
        self.palette.width() as u32
    }
}

impl PaletteQuantization {
    fn into_quantizer<P>(self) -> impl Quantizer<P, P>
    where
        P: Pixel + std::ops::Sub<Output = P> + FromFlat,
        RGB: ColorSpace<P>,
        BoundError: ErrorCombinator<P>,
    {
        let ndim = NDimImage::new(self.palette.shape(), self.palette.data().to_vec());
        let img: Image<P> = ndim
            .into_pixels()
            .expect("Expected shape of palette to match.");

        ColorPalette::new(RGB, img.take(), BoundError)
    }
}

#[derive(FromPyObject)]
pub enum Quant {
    Uniform(UniformQuantization),
    Palette(PaletteQuantization),
}

#[pyclass]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DiffusionAlgorithm {
    FloydSteinberg = 0,
    JarvisJudiceNinke = 1,
    Stucki = 2,
    Atkinson = 3,
    Burkes = 4,
    Sierra = 5,
    TwoRowSierra = 6,
    SierraLite = 7,
}

#[pyfunction]
pub fn quantize<'py>(
    py: Python<'py>,
    img: PyImage<'py>,
    quant: Quant,
) -> PyResult<&'py PyArray3<f32>> {
    match quant {
        Quant::Uniform(quant) => {
            let mut img: NDimImage = img.load_image()?;
            let result = py.allow_threads(|| {
                image_ops::dither::quantize_ndim(&mut img, quant.inner);
                img.into_numpy()
            });
            Ok(result.into_pyarray(py))
        }
        Quant::Palette(quant) => {
            fn with_pixel_format<'py, P>(
                py: Python<'py>,
                img: PyImage<'py>,
                quant: impl Quantizer<P, P> + Sync,
            ) -> PyResult<&'py PyArray3<f32>>
            where
                P: Pixel + Send + FromFlat,
                Image<P>: IntoNumpy,
            {
                let mut img: Image<P> = img.load_image()?;
                let result = py.allow_threads(|| {
                    image_ops::dither::quantize(&mut img, &quant);
                    img.into_numpy()
                });
                Ok(result.into_pyarray(py))
            }

            let c = img.channels();
            match c {
                1 => with_pixel_format::<f32>(py, img, quant.into_quantizer()),
                3 => with_pixel_format::<Vec3A>(py, img, quant.into_quantizer()),
                4 => with_pixel_format::<Vec4>(py, img, quant.into_quantizer()),
                _ => Err(PyValueError::new_err(format!(
                        "Argument '{}' does not have the right shape. Expected 1, 3, or 4 channels but found {}.",
                        stringify!(img),
                        c
                    ))),
            }
        }
    }
}

#[pyfunction]
pub fn ordered_dither<'py>(
    py: Python<'py>,
    img: PyImage,
    quant: UniformQuantization,
    map_size: u32,
) -> PyResult<&'py PyArray3<f32>> {
    if !map_size.is_power_of_two() {
        return Err(PyValueError::new_err(format!(
            "Argument '{}' must be a power of 2.",
            stringify!(map_size)
        )));
    }

    let mut img = img.load_image()?;
    let result = py.allow_threads(|| {
        image_ops::dither::ordered_dither(&mut img, map_size as usize, quant.inner);
        img.into_numpy()
    });
    Ok(result.into_pyarray(py))
}

mod diffusion {
    use image_core::FromFlat;

    use super::*;

    pub struct Config<'py>(pub Python<'py>, pub PyImage<'py>);

    fn with_pixel_format<P>(
        Config(py, img): Config<'_>,
        quant: impl Quantizer<P, P> + Sync,
        algorithm: impl image_ops::dither::DiffusionAlgorithm + Send,
    ) -> PyResult<&PyArray3<f32>>
    where
        P: Pixel + Send + FromFlat,
        Image<P>: IntoNumpy,
    {
        let mut img: Image<P> = img.load_image()?;
        let result = py.allow_threads(|| {
            image_ops::dither::error_diffusion_dither(&mut img, algorithm, &quant);
            img.into_numpy()
        });
        Ok(result.into_pyarray(py))
    }

    pub fn with_algorithm(
        config: Config,
        quant: Quant,
        algorithm: impl image_ops::dither::DiffusionAlgorithm + Send,
    ) -> PyResult<&PyArray3<f32>> {
        let c = config.1.channels();
        let err = Err(PyValueError::new_err(format!(
            "Argument '{}' does not have the right shape. Expected 1, 3, or 4 channels but found {}.",
            stringify!(img),
            c
        )));

        match quant {
            Quant::Uniform(quant) => match c {
                1 => with_pixel_format::<f32>(config, quant.inner, algorithm),
                3 => with_pixel_format::<Vec3A>(config, quant.inner, algorithm),
                4 => with_pixel_format::<Vec4>(config, quant.inner, algorithm),
                _ => err,
            },
            Quant::Palette(quant) => match c {
                1 => with_pixel_format::<f32>(config, quant.into_quantizer(), algorithm),
                3 => with_pixel_format::<Vec3A>(config, quant.into_quantizer(), algorithm),
                4 => with_pixel_format::<Vec4>(config, quant.into_quantizer(), algorithm),
                _ => err,
            },
        }
    }
}

#[pyfunction]
pub fn error_diffusion_dither<'py>(
    py: Python<'py>,
    img: PyImage<'py>,
    quant: Quant,
    algorithm: DiffusionAlgorithm,
) -> PyResult<&'py PyArray3<f32>> {
    use diffusion::*;

    let config: Config<'py> = Config(py, img);
    match algorithm {
        DiffusionAlgorithm::FloydSteinberg => with_algorithm(config, quant, FloydSteinberg),
        DiffusionAlgorithm::JarvisJudiceNinke => with_algorithm(config, quant, JarvisJudiceNinke),
        DiffusionAlgorithm::Stucki => with_algorithm(config, quant, Stucki),
        DiffusionAlgorithm::Atkinson => with_algorithm(config, quant, Atkinson),
        DiffusionAlgorithm::Burkes => with_algorithm(config, quant, Burkes),
        DiffusionAlgorithm::Sierra => with_algorithm(config, quant, Sierra),
        DiffusionAlgorithm::TwoRowSierra => with_algorithm(config, quant, TwoRowSierra),
        DiffusionAlgorithm::SierraLite => with_algorithm(config, quant, SierraLite),
    }
}

mod riemersma {
    use image_core::FromFlat;

    use crate::convert::LoadImage;

    use super::*;

    pub struct Config<'py>(pub Python<'py>, pub PyImage<'py>, pub usize, pub f32);

    pub fn with_pixel_format<P>(
        Config(py, img, history_length, decay_ratio): Config<'_>,
        quant: impl Quantizer<P, P> + Sync,
    ) -> PyResult<&PyArray3<f32>>
    where
        P: Pixel + Send + FromFlat,
        Image<P>: IntoNumpy,
    {
        let mut img: Image<P> = img.load_image()?;
        let result = py.allow_threads(|| {
            image_ops::dither::riemersma_dither(&mut img, history_length, decay_ratio, &quant);
            img.into_numpy()
        });
        Ok(result.into_pyarray(py))
    }
}

#[pyfunction]
pub fn riemersma_dither<'py>(
    py: Python<'py>,
    img: PyImage<'py>,
    quant: Quant,
    history_length: u32,
    decay_ratio: f32,
) -> PyResult<&'py PyArray3<f32>> {
    if history_length < 2 {
        return Err(PyValueError::new_err(format!(
            "Argument '{}' must be at least 2.",
            stringify!(history_length)
        )));
    }

    use riemersma::*;

    let c = img.channels();
    let config: Config<'py> = Config(py, img, history_length as usize, decay_ratio);
    let err = PyValueError::new_err(format!(
        "Argument '{}' does not have the right shape. Expected 1, 3, or 4 channels but found {}.",
        stringify!(img),
        c
    ));

    match quant {
        Quant::Uniform(quant) => match c {
            1 => with_pixel_format::<f32>(config, quant.inner),
            3 => with_pixel_format::<Vec3A>(config, quant.inner),
            4 => with_pixel_format::<Vec4>(config, quant.inner),
            _ => Err(err),
        },
        Quant::Palette(quant) => match c {
            1 => with_pixel_format::<f32>(config, quant.into_quantizer()),
            3 => with_pixel_format::<Vec3A>(config, quant.into_quantizer()),
            4 => with_pixel_format::<Vec4>(config, quant.into_quantizer()),
            _ => Err(err),
        },
    }
}
