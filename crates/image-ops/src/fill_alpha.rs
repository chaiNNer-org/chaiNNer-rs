use glam::Vec4;
use image_core::Image;

use crate::{
    blend::{overlay_mut, overlay_self_mut},
    fragment_blur::fragment_blur_alpha,
    util::from_image_cow,
};

pub enum FillMode {
    Texture,
    Color,
}

pub fn fill_alpha(
    image: &mut Image<Vec4>,
    threshold: f32,
    mode: FillMode,
    temp: Option<&mut Image<Vec4>>,
) {
    make_binary_alpha(image.data_mut(), threshold);

    match mode {
        FillMode::Texture => fill_alpha_fragment_blur(image, temp),
        FillMode::Color => todo!(),
    }
}

fn make_binary_alpha(pixels: &mut [Vec4], threshold: f32) {
    for p in pixels {
        let a: f32 = if p.w < threshold { 0. } else { 1. };
        *p *= a;
        p.w = a;
    }
}

fn fill_alpha_fragment_blur(image: &mut Image<Vec4>, temp: Option<&mut Image<Vec4>>) {
    let original = &*from_image_cow(&image, temp);
    let mut buffer: Image<Vec4> = Image::from_const(image.size(), Vec4::ZERO);

    for i in 0..6 {
        let radius = (1 << i) as f32;
        let angle_offset = i as f32;

        buffer = fragment_blur_alpha(&original, radius, 5, angle_offset, Some(buffer));
        overlay_self_mut(&mut buffer, 2);
        overlay_mut(&mut buffer, &image);
        std::mem::swap(&mut buffer, image);
    }

    make_binary_alpha(image.data_mut(), 0.01);
}

#[cfg(test)]
mod tests {
    use test_util::{data::read_flower_transparent, snap::ImageSnapshot};

    #[test]
    fn fill_alpha_texture() {
        let mut original = read_flower_transparent();
        super::fill_alpha(&mut original, 0.15, super::FillMode::Texture, None);
        original.snapshot("fill_alpha_texture");
    }
}
