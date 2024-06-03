use std::io::Error;

use image::{io::Reader as ImageReader, ColorType};

use crate::NDimImage;
use crate::Shape;

pub fn load_image(path: &str) -> Result<NDimImage, Error> {
    let img = ImageReader::open(path)
        .expect("Unable to find image")
        .decode()
        .expect("Unable to decode image");
    let channels: usize = match img.color() {
        ColorType::Rgba32F => 4,
        ColorType::Rgba16 => 4,
        ColorType::Rgba8 => 4,
        ColorType::Rgb32F => 3,
        ColorType::Rgb16 => 3,
        ColorType::Rgb8 => 3,
        ColorType::L16 => 1,
        ColorType::L8 => 1,
        _ => panic!("Unsupported number of channels"),
    };
    let img_shape = Shape::new(img.width() as usize, img.height() as usize, channels);
    let converted_img = match channels {
        4 => img.to_rgba32f().to_vec(),
        3 => img.to_rgb32f().to_vec(),
        1 => img.to_luma32f().to_vec(),
        _ => panic!("Unsupported number of channels"),
    };
    let ndarray = NDimImage::new(img_shape, converted_img);
    return Ok(ndarray);
}
