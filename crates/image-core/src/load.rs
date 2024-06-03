use image::io::Reader as ImageReader;

use crate::NDimImage;
use crate::Shape;
use crate::Size;

pub fn load_image(path: &str) -> Result<NDimImage, Box<dyn std::error::Error>> {
    let img = ImageReader::open(path)?.decode()?;

    let size = Size::new(img.width() as usize, img.height() as usize);

    let channels = img.color().channel_count();
    let f32_data = match channels {
        1 => img.to_luma32f().into_raw(),
        3 => img.into_rgb32f().into_raw(),
        _ => img.into_rgba32f().into_raw(),
    };

    assert!(f32_data.len() % size.len() == 0);
    let channels = f32_data.len() / size.len();
    let img_shape = Shape::from_size(size, channels);

    Ok(NDimImage::new(img_shape, f32_data))
}
