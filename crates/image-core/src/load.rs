use image::io::Reader as ImageReader;
use image::DynamicImage;

use crate::util::slice_as_chunks;
use crate::NDimImage;
use crate::Shape;
use crate::Size;

trait Normalize {
    fn normalize(self) -> f32;
}
impl Normalize for u8 {
    fn normalize(self) -> f32 {
        self as f32 * (1.0 / 255.0)
    }
}
impl Normalize for u16 {
    fn normalize(self) -> f32 {
        self as f32 * (1.0 / 65535.0)
    }
}
impl Normalize for f32 {
    fn normalize(self) -> f32 {
        self
    }
}

fn rgba_to_bgra<T>(data: &[T]) -> Vec<f32>
where
    T: Copy + Normalize,
{
    assert!(data.len() % 4 == 0, "data length must be multiple of 4");
    let mut out = Vec::with_capacity(data.len());

    let (chunks, rest) = slice_as_chunks::<T, 4>(data);
    assert!(rest.is_empty(), "data length must be multiple of 4");
    for chunk in chunks {
        out.extend_from_slice(&[
            chunk[2].normalize(),
            chunk[1].normalize(),
            chunk[0].normalize(),
            chunk[3].normalize(),
        ]);
    }
    out
}

fn rgb_to_bgr<T>(data: &[T]) -> Vec<f32>
where
    T: Copy + Normalize,
{
    assert!(data.len() % 3 == 0, "data length must be multiple of 3");
    let mut out = Vec::with_capacity(data.len());

    let (chunks, rest) = slice_as_chunks::<T, 12>(data);
    for chunk in chunks {
        out.extend_from_slice(&[
            chunk[2].normalize(),
            chunk[1].normalize(),
            chunk[0].normalize(),
            chunk[5].normalize(),
            chunk[4].normalize(),
            chunk[3].normalize(),
            chunk[8].normalize(),
            chunk[7].normalize(),
            chunk[6].normalize(),
            chunk[11].normalize(),
            chunk[10].normalize(),
            chunk[9].normalize(),
        ]);
    }

    // now the rest
    let (chunks, rest) = slice_as_chunks::<T, 3>(rest);
    assert!(rest.is_empty(), "data length must be multiple of 3");
    for chunk in chunks {
        out.extend_from_slice(&[
            chunk[2].normalize(),
            chunk[1].normalize(),
            chunk[0].normalize(),
        ]);
    }

    out
}

pub fn load_image(path: &str) -> Result<NDimImage, Box<dyn std::error::Error>> {
    let img = ImageReader::with_guessed_format(ImageReader::open(path)?)?.decode()?;

    let size = Size::new(img.width() as usize, img.height() as usize);

    let img_data: Vec<f32> = match img {
        DynamicImage::ImageLuma8(_) | DynamicImage::ImageLuma16(_) => img.to_luma32f().into_raw(),
        DynamicImage::ImageLumaA8(_) | DynamicImage::ImageLumaA16(_) => {
            img.into_rgba32f().into_raw()
        }
        DynamicImage::ImageRgb8(img) => rgb_to_bgr(img.as_ref()),
        DynamicImage::ImageRgba8(img) => rgba_to_bgra(img.as_ref()),
        DynamicImage::ImageRgb16(img) => rgb_to_bgr(img.as_ref()),
        DynamicImage::ImageRgba16(img) => rgba_to_bgra(img.as_ref()),
        DynamicImage::ImageRgb32F(img) => rgb_to_bgr(img.as_ref()),
        DynamicImage::ImageRgba32F(img) => rgba_to_bgra(img.as_ref()),
        _ => return Err("Unsupported image format".into()),
    };

    assert!(img_data.len() % size.len() == 0);
    let channels = img_data.len() / size.len();
    let img_shape = Shape::from_size(size, channels);

    Ok(NDimImage::new(img_shape, img_data))
}
