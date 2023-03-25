use std::{
    f32::consts::PI,
    ops::{AddAssign, DivAssign, MulAssign, Range},
};

use glam::Vec4;
use image_core::Image;

use crate::util::{from_const, move_range, move_range_i};

struct Offset(isize, isize);

fn get_offsets(radius: f32, count: usize, angle_offset: f32) -> Vec<Offset> {
    assert!(count >= 1);

    (0..count)
        .map(|i| {
            let angle = i as f32 / count as f32 * PI * 2. + angle_offset;
            let x = (angle.sin() * radius).round() as isize;
            let y = (angle.cos() * radius).round() as isize;
            Offset(x, y)
        })
        .collect()
}

/// Returns a range such that all values in the range `+ offset` are in the range `0..len`.
fn offset_range(offset: isize, len: usize) -> Range<usize> {
    let start = (-offset).clamp(0, len as isize) as usize;
    let end = (len as isize - offset).clamp(0, len as isize) as usize;
    start..end
}

/// Applies fragment blur to the given image.
///
/// This method assumes that the given image has premultiplied alpha.
pub fn fragment_blur_premultiplied_alpha(
    src: &Image<Vec4>,
    radius: f32,
    count: usize,
    angle_offset: f32,
    out: Option<Image<Vec4>>,
) -> Image<Vec4> {
    let mut dest = from_const(src.size(), Vec4::ZERO, out);
    let w = src.width();
    let h = src.height();

    let s_data = src.data();
    let d = dest.data_mut();

    assert!(count <= 255);
    let mut count_array: Vec<u8> = vec![0; d.len()];

    for Offset(offset_x, offset_y) in get_offsets(radius, count, angle_offset) {
        let x_range = offset_range(offset_x, w);
        let y_range = offset_range(offset_y, h);
        if x_range.is_empty() || y_range.is_empty() {
            continue;
        }
        let index_offset = offset_y * w as isize + offset_x;
        for y in y_range {
            let dest_range = move_range(&x_range, y * w);
            let src_range = move_range_i(&dest_range, index_offset);

            let src_data = &s_data[src_range];
            let dst_data = &mut d[dest_range.clone()];
            assert_eq!(src_data.len(), dst_data.len());
            for (d, s) in dst_data.iter_mut().zip(src_data) {
                *d += *s;
            }

            for c in &mut count_array[dest_range] {
                *c += 1;
            }
        }
    }

    for (p, c) in d.iter_mut().zip(count_array) {
        let rgb = if p.w == 0. { 1. } else { 1. / p.w };
        let a = if c == 0 { 1. } else { 1. / c as f32 };
        p.mul_assign(Vec4::new(rgb, rgb, rgb, a));
    }

    dest
}

/// Applies fragment blur to the given image.
pub fn fragment_blur_alpha(
    src: &Image<Vec4>,
    radius: f32,
    count: usize,
    angle_offset: f32,
    out: Option<Image<Vec4>>,
) -> Image<Vec4> {
    let pre = src.map(|v| Vec4::new(v.x * v.w, v.y * v.w, v.z * v.w, v.w));
    fragment_blur_premultiplied_alpha(&pre, radius, count, angle_offset, out)
}

/// Applies fragment blur to the given image.
///
/// Each channel will be blurred independently of each other. If the image has an alpha channel,
/// use [`fragment_blur_alpha`] instead.
pub fn fragment_blur<P>(
    src: &Image<P>,
    radius: f32,
    count: usize,
    angle_offset: f32,
    out: Option<Image<P>>,
) -> Image<P>
where
    P: Clone + Default + AddAssign + DivAssign<f32>,
{
    let mut dest = from_const(src.size(), Default::default(), out);
    let w = src.width();
    let h = src.height();

    let s_data = src.data();
    let d = dest.data_mut();

    assert!(count <= 255);
    let mut count_array: Vec<u8> = vec![0; d.len()];

    for Offset(offset_x, offset_y) in get_offsets(radius, count, angle_offset) {
        let x_range = offset_range(offset_x, w);
        let y_range = offset_range(offset_y, h);
        if x_range.is_empty() || y_range.is_empty() {
            continue;
        }
        let index_offset = offset_y * w as isize + offset_x;
        for y in y_range {
            let dest_range = move_range(&x_range, y * w);
            let src_range = move_range_i(&dest_range, index_offset);

            let src_data = &s_data[src_range];
            let dst_data = &mut d[dest_range.clone()];
            assert_eq!(src_data.len(), dst_data.len());
            for (d, s) in dst_data.iter_mut().zip(src_data) {
                *d += s.clone();
            }

            for c in &mut count_array[dest_range] {
                *c += 1;
            }
        }
    }

    // set all count==0 to 1
    for c in &mut count_array {
        *c = if *c == 0 { 1 } else { *c }
    }

    let c = &count_array[..];
    assert!(c.len() == d.len());
    for i in 0..d.len() {
        d[i].div_assign(c[i] as f32);
    }

    dest
}

#[cfg(test)]
mod tests {
    use test_util::{
        data::{read_abstract_transparent, read_flower_transparent, read_portrait},
        snap::ImageSnapshot,
    };

    #[test]
    fn fragment_blur() {
        let original = read_portrait();
        let result = super::fragment_blur(&original, 20., 5, 1.234, None);
        result.snapshot("fragment_blur");
    }

    #[test]
    fn fragment_blur_alpha() {
        let original = read_flower_transparent();
        let result = super::fragment_blur_alpha(&original, 20., 5, 1.234, None);
        result.snapshot("fragment_blur_alpha-1");

        let original = read_abstract_transparent();
        let result = super::fragment_blur_alpha(&original, 20., 5, 1.234, None);
        result.snapshot("fragment_blur_alpha-2");
    }
}
