use glam::{Vec3A, Vec4};
use image::{io::Reader as ImageReader, DynamicImage};
use image_core::{util::slice_as_chunks, Image, Size};

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
    let (chunks, rest) = slice_as_chunks::<_, 4>(&image);
    assert!(rest.is_empty());
    let data = chunks
        .iter()
        .map(|[r, g, b, a]| Vec4::new(*r, *g, *b, *a))
        .collect();
    Image::new(size, data)
}
fn into_vec3(image: DynamicImage) -> Image<Vec3A> {
    let image = image.into_rgb32f();
    let size = Size::new(image.width() as usize, image.height() as usize);
    let (chunks, rest) = slice_as_chunks::<_, 3>(&image);
    assert!(rest.is_empty());
    let data = chunks
        .iter()
        .map(|[r, g, b]| Vec3A::new(*r, *g, *b))
        .collect();
    Image::new(size, data)
}
fn into_scalar(image: DynamicImage) -> Image<f32> {
    let image = image.into_rgb32f();
    let size = Size::new(image.width() as usize, image.height() as usize);
    let (chunks, rest) = slice_as_chunks::<_, 3>(&image);
    assert!(rest.is_empty());
    let data = chunks.iter().map(|[r, _, _]| *r).collect();
    Image::new(size, data)
}

pub fn read_lion() -> Image<Vec3A> {
    into_vec3(read_image!("lion.png"))
}
pub fn read_flower() -> Image<Vec3A> {
    into_vec3(read_image!("flower.png"))
}
pub fn read_flower_palette() -> Image<Vec3A> {
    into_vec3(read_image!("flower-palette.png"))
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
pub fn read_at_sdf() -> Image<Vec3A> {
    into_vec3(read_image!("at-sdf.png"))
}
pub fn read_at() -> Image<f32> {
    into_scalar(read_image!("at.png"))
}
pub fn read_checker() -> Image<f32> {
    into_scalar(read_image!("checker.png"))
}
pub fn read_binary_alpha() -> Image<f32> {
    into_scalar(read_image!("binary-alpha.png"))
}
pub fn read_nes_smb() -> Image<Vec3A> {
    into_vec3(read_image!("nes-smb.png"))
}
