use image_core::{Image, ImageView, Size};

use super::{Filter, FloatPixelFormat, PixelFormat};

pub fn scale<P>(img: ImageView<P>, size: Size, filter: Filter) -> Result<Image<P>, resize::Error>
where
    P: Clone + Default,
    FloatPixelFormat<P>: PixelFormat<InputPixel = P, OutputPixel = P>,
{
    if size.is_empty() {
        return Ok(Image::new(size, Vec::new()));
    }

    let filter_type = match filter {
        Filter::Nearest => {
            // the nearest implementation isn't correct, so we use our own
            return Ok(nearest_neighbor(img, size));
        }
        _ => filter.into(),
    };

    let mut dest = Image::from_const(size, P::default());

    resize::Resizer::new(
        img.width(),
        img.height(),
        size.width,
        size.height,
        FloatPixelFormat::default(),
        filter_type,
    )?
    .resize(img.data(), dest.data_mut())?;

    Ok(dest)
}

fn nearest_neighbor<P: Clone>(src: ImageView<P>, size: Size) -> Image<P> {
    if src.size() == size {
        return src.into_owned();
    }

    let src_size = src.size();
    let src = src.data();

    {
        // optimization for power-of-2 scaling factors, e.g. 2x, 4x
        let scale_up = size.width / src_size.width;
        if size == src_size.scale(scale_up as f64) && scale_up.is_power_of_two() {
            let shift = scale_up.trailing_zeros();

            let mut data = Vec::with_capacity(size.len());
            for y in 0..size.height {
                let src_y = y >> shift;
                let src_i = src_y * src_size.width;

                data.extend((0..size.width).map(move |x| {
                    let src_x = x >> shift;
                    src[src_i + src_x].clone()
                }));
            }

            return Image::new(size, data);
        }
    }

    // What is going on here? Okay, so this uses fixed point arithmetic (fixed)
    // to avoid floating point and divisions. Basic NN works like this:
    // We imagine that each pixel coordinate is at the center of the pixel and that center coordinate is then mapped to the src image. For the x coordinate this means:
    //
    //   src_x = round((x_index + 0.5) * src_width / width - 0.5)
    //         = floor((x_index + 0.5) * src_width / width)
    //
    // Let's define `k = src_width / width`.
    //
    //   src_x = floor((x_index + 0.5) * k)
    //         = floor(x_index * k + k/2)
    //
    // Now the fixed comes in. In fixed, `floor(x)` is just a bit shift, it's
    // super cheap.
    const SHIFT: i32 = 32;

    assert!(src_size.width <= i32::MAX as usize);
    assert!(src_size.height <= i32::MAX as usize);

    let k_x: u64 = ((src_size.width as u64) << SHIFT) / size.width as u64;
    let k_y: u64 = ((src_size.height as u64) << SHIFT) / size.height as u64;
    let k_x_half: u64 = k_x >> 1;
    let k_y_half: u64 = k_y >> 1;

    let mut data = Vec::with_capacity(size.len());
    for y in 0..(size.height as u64) {
        let src_y = ((y * k_y + k_y_half) >> SHIFT) as usize;
        let src_i = src_y * src_size.width;

        data.extend((0..(size.width as u64)).map(move |x| {
            let src_x = ((x * k_x + k_x_half) >> SHIFT) as usize;
            src[src_i + src_x].clone()
        }));
    }

    Image::new(size, data)
}

#[cfg(test)]
mod tests {
    use glam::Vec3A;
    use image_core::Size;
    use test_util::{data::read_portrait, snap::ImageSnapshot};

    fn small_portrait() -> image_core::Image<Vec3A> {
        let img = read_portrait();
        super::scale(img.view(), img.size().scale(0.5), super::Filter::Linear).unwrap()
    }

    #[test]
    fn scale_nearest() {
        let filter = super::Filter::Nearest;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_nearest_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_nearest_200");
    }

    #[test]
    fn scale_box() {
        let filter = super::Filter::Box;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_box_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_box_200");
    }

    #[test]
    fn scale_linear() {
        let filter = super::Filter::Linear;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_linear_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_linear_200");
    }

    #[test]
    fn scale_hermite() {
        let filter = super::Filter::Hermite;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_hermite_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_hermite_200");
    }

    #[test]
    fn scale_cubic_catrom() {
        let filter = super::Filter::CubicCatrom;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_cubic_catrom_4x");

        // https://github.com/chaiNNer-org/chaiNNer-rs/pull/20#issuecomment-1839525313
        // let original = read_portrait();
        // let new_size = Size::new(200, 200);
        // let nn = super::scale(original.view(), new_size, filter).unwrap();
        // nn.snapshot("resize_cubic_catrom_200");
    }

    #[test]
    fn scale_cubic_bspline() {
        let filter = super::Filter::CubicBSpline;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_cubic_bspline_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_cubic_bspline_200");
    }

    #[test]
    fn scale_cubic_mitchell() {
        let filter = super::Filter::CubicMitchell;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_cubic_mitchell_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_cubic_mitchell_200");
    }

    #[test]
    fn scale_hamming() {
        let filter = super::Filter::Hamming;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_hamming_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_hamming_200");
    }

    #[test]
    fn scale_hann() {
        let filter = super::Filter::Hann;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_hann_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_hann_200");
    }

    #[test]
    fn scale_lanczos3() {
        let filter = super::Filter::Lanczos3;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_lanczos3_4x");

        // https://github.com/chaiNNer-org/chaiNNer-rs/pull/20#issuecomment-1839525313
        // let original = read_portrait();
        // let new_size = Size::new(200, 200);
        // let nn = super::scale(original.view(), new_size, filter).unwrap();
        // nn.snapshot("resize_lanczos3_200");
    }

    #[test]
    fn scale_gauss() {
        let filter = super::Filter::Gauss;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_gauss_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_gauss_200");
    }

    #[test]
    fn scale_lagrange() {
        let filter = super::Filter::Lagrange;

        let original = small_portrait();
        let new_size = original.size().scale(4.);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_lagrange_4x");

        let original = read_portrait();
        let new_size = Size::new(200, 200);
        let nn = super::scale(original.view(), new_size, filter).unwrap();
        nn.snapshot("resize_lagrange_200");
    }
}
