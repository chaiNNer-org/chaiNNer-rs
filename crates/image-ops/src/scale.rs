use image_core::{Image, Size};

pub fn nearest_neighbor<P: Clone>(src: &Image<P>, size: Size) -> Image<P> {
    if src.size() == size {
        return src.clone();
    }

    let src_size = src.size();
    let src = src.data();

    {
        let scale_up = size.width / src_size.width;
        if size == src_size.scale(scale_up as f64) {
            // the new size is a constant factor upscale of the original
            return Image::new(
                size,
                size.iter_pos()
                    .map(|(x, y)| {
                        let src_x = x / scale_up;
                        let src_y = y / scale_up;
                        src[src_y * src_size.width + src_x].clone()
                    })
                    .collect(),
            );
        }
    }

    let x_off_center = src_size.width / 2;
    let y_off_center = src_size.height / 2;
    let new_data = size
        .iter_pos()
        .map(|(x, y)| {
            // Why this off center term?
            // We imagine that each pixel coordinate is at the center of the pixel and that center coordinate is then mapped to the src image. For the x coordinate this means:
            //   src_x = round((x_index + 0.5) * src_width / width - 0.5)
            //         = floor((x_index + 0.5) * src_width / width)
            //         = ((x_index + 0.5) * src_width) // width
            //         = (x_index*src_width + src_width/2) // width
            //        ~= (x_index*src_width + src_width//2) // width
            let src_x = (x * src_size.width + x_off_center) / size.width;
            let src_y = (y * src_size.height + y_off_center) / size.height;
            src[src_y * src_size.width + src_x].clone()
        })
        .collect();

    Image::new(size, new_data)
}

#[cfg(test)]
mod tests {
    use image_core::Size;
    use test_util::{data::read_portrait, snap::ImageSnapshot};

    #[test]
    fn nearest_neighbor() {
        let original = read_portrait();

        let new_size = original.size().scale(4.);
        let nn = super::nearest_neighbor(&original, new_size);
        assert_eq!(nn.size(), new_size);
        nn.snapshot("nearest_neighbor_4x");

        let new_size = Size::new(200, 200);
        let nn = super::nearest_neighbor(&original, new_size);
        assert_eq!(nn.size(), new_size);
        nn.snapshot("nearest_neighbor_200");
    }
}
