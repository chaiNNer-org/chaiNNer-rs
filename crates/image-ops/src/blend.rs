use glam::Vec4;
use image_core::Image;

/// Overlays the given image with itself `n` times.
///
/// This operation is a noop for n < 2.
pub fn overlay_self_mut(img: &mut Image<Vec4>, n: u32) {
    if n < 2 {
        return;
    }

    if n == 2 {
        for p in img.data_mut() {
            let a_i = 1. - p.w;
            p.w = 1. - a_i * a_i;
        }
    } else if n == 3 {
        for p in img.data_mut() {
            let a_i = 1. - p.w;
            p.w = 1. - a_i * a_i * a_i;
        }
    } else {
        for p in img.data_mut() {
            let a_i = 1. - p.w;
            p.w = 1. - a_i.powi(n.try_into().expect("n is too large"));
        }
    }
}

pub fn overlay_mut(img: &mut Image<Vec4>, top: &Image<Vec4>) {
    assert!(img.size() == top.size());

    let data = img.data_mut();
    assert!(data.len() == top.len());

    for (i, a) in data.iter_mut().enumerate() {
        let b = top.data()[i];

        let final_alpha = 1. - (1. - a.w) * (1. - b.w);

        let mut rgb = b * b.w + *a * a.w * (1. - b.w);
        rgb /= if final_alpha == 0. { 1. } else { final_alpha };
        *a = rgb;
        a.w = final_alpha;
    }
}

#[cfg(test)]
mod tests {
    use test_util::{data::read_abstract_transparent, snap::ImageSnapshot};

    #[test]
    fn overlay_self_mut() {
        let mut original = read_abstract_transparent();
        super::overlay_self_mut(&mut original, 2);
        original.snapshot("overlay_self_mut");
    }

    #[test]
    fn overlay_mut() {
        let mut original = read_abstract_transparent();
        let other = original.clone();
        super::overlay_mut(&mut original, &other);
        original.snapshot("overlay_mut");
    }
}
