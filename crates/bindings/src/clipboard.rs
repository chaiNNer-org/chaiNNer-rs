use image_core::{
    util::{slice_as_chunks, vec_into_flattened},
    NDimCow, NDimView,
};
use pyo3::{exceptions::PyValueError, prelude::*};
use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
};

use crate::convert::{LoadImage, PyImage};

#[pyclass(frozen)]
pub struct Clipboard {
    inner: Arc<Mutex<arboard::Clipboard>>,
}

impl Clipboard {
    fn get_clipboard(&self) -> PyResult<std::sync::MutexGuard<'_, arboard::Clipboard>> {
        self.inner
            .lock()
            .map_err(|e| PyValueError::new_err(format!("Failed to lock clipboard: {}", e)))
    }
}

#[pymethods]
impl Clipboard {
    pub fn write_text(&self, text: String) -> PyResult<()> {
        self.get_clipboard()?
            .set_text(text)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    pub fn write_image(&self, image: PyImage, pixel_format: &str) -> PyResult<()> {
        let pixel_format = match pixel_format {
            "RGB" => PixelFormat::Rgb,
            "BGR" => PixelFormat::Bgr,
            _ => {
                return Err(PyValueError::new_err(format!(
                    "Invalid pixel format: {}",
                    pixel_format
                )));
            }
        };

        let image: NDimCow = image.load_image()?;
        let image = to_image_data(image.view(), pixel_format)?;

        self.get_clipboard()?
            .set_image(image)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    pub fn create_instance() -> PyResult<Self> {
        let inner = arboard::Clipboard::new()
            .map_err(|e| PyValueError::new_err(format!("Failed to create clipboard: {}", e)))?;

        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }
}

enum PixelFormat {
    Rgb,
    Bgr,
}

fn to_image_data(
    image: NDimView,
    pixel_format: PixelFormat,
) -> PyResult<arboard::ImageData<'static>> {
    let width = image.width();
    let height = image.height();
    let channels = image.channels();

    fn to_u8(v: f32) -> u8 {
        (v * 255. + 0.5).floor().clamp(0.0, 255.0) as u8
    }

    let bytes: Vec<[u8; 4]> = match channels {
        1 => image
            .data()
            .iter()
            .map(|v| {
                let v = to_u8(*v);
                [v, v, v, 255]
            })
            .collect(),
        3 => {
            let (data, rest) = slice_as_chunks::<f32, 3>(image.data());
            assert!(rest.is_empty());

            match pixel_format {
                PixelFormat::Rgb => data
                    .iter()
                    .map(|[r, g, b]| [to_u8(*r), to_u8(*g), to_u8(*b), 255])
                    .collect(),
                PixelFormat::Bgr => data
                    .iter()
                    .map(|[b, g, r]| [to_u8(*r), to_u8(*g), to_u8(*b), 255])
                    .collect(),
            }
        }
        4 => {
            let (data, rest) = slice_as_chunks::<f32, 4>(image.data());
            assert!(rest.is_empty());

            match pixel_format {
                PixelFormat::Rgb => data
                    .iter()
                    .map(|[r, g, b, a]| [to_u8(*r), to_u8(*g), to_u8(*b), to_u8(*a)])
                    .collect(),
                PixelFormat::Bgr => data
                    .iter()
                    .map(|[b, g, r, a]| [to_u8(*r), to_u8(*g), to_u8(*b), to_u8(*a)])
                    .collect(),
            }
        }
        _ => {
            return Err(PyValueError::new_err(format!(
                "Invalid number of channels: {}",
                channels
            )));
        }
    };

    let image = arboard::ImageData {
        width,
        height,
        bytes: Cow::Owned(vec_into_flattened(bytes)),
    };

    Ok(image)
}
