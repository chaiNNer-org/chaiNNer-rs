use glam::{Vec3A, Vec4};
use image::{io::Reader as ImageReader, DynamicImage};
use image_core::{Image, Size};

macro_rules! read_image {
    ($name:literal) => {{
        let path = concat!("../../data/", $name);
        ImageReader::open(path)
            .expect(concat!("Unable to find image ", $name))
            .decode()
            .expect(concat!("Unable to decode image ", $name))
    }};
}

fn into_vec4(image: DynamicImage) -> Image<Vec4> {
    let image = image.into_rgba32f();
    let size = Size::new(image.width() as usize, image.height() as usize);
    let (chunks, rest) = image.as_chunks::<4>();
    assert!(rest.is_empty());
    let data = chunks
        .into_iter()
        .map(|[r, g, b, a]| Vec4::new(*r, *g, *b, *a))
        .collect();
    Image::new(size, data)
}
fn into_vec3(image: DynamicImage) -> Image<Vec3A> {
    let image = image.into_rgb32f();
    let size = Size::new(image.width() as usize, image.height() as usize);
    let (chunks, rest) = image.as_chunks::<3>();
    assert!(rest.is_empty());
    let data = chunks
        .into_iter()
        .map(|[r, g, b]| Vec3A::new(*r, *g, *b))
        .collect();
    Image::new(size, data)
}

pub fn read_lion() -> Image<Vec3A> {
    into_vec3(read_image!("lion.png"))
}
pub fn read_flower() -> Image<Vec3A> {
    into_vec3(read_image!("flower.png"))
}
pub fn read_portrait() -> Image<Vec3A> {
    into_vec3(read_image!("portrait.png"))
}
pub fn read_flower_transparent() -> Image<Vec4> {
    into_vec4(read_image!("flower-transparent.png"))
}
pub fn read_abstract_transparent() -> Image<Vec4> {
    into_vec4(read_image!("abstract-transparent.png"))
}
