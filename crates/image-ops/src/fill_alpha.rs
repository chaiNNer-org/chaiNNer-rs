use glam::Vec4;
use image_core::{Image, Size};
use rstar::{primitives::GeomWithData, RTree};
use std::ops::Range;

use crate::{
    blend::{overlay_mut, overlay_self_mut},
    fragment_blur::fragment_blur_alpha,
    util::{div_ceil, from_image_cow, move_range, Grid},
};

pub enum FillMode {
    Fragment {
        iterations: u32,
        fragment_count: u32,
    },
    ExtendColor {
        iterations: u32,
    },
    Nearest {
        min_radius: u32,
        anti_aliasing: bool,
    },
}

pub fn fill_alpha(
    image: &mut Image<Vec4>,
    threshold: f32,
    mode: FillMode,
    temp: Option<&mut Image<Vec4>>,
) {
    make_binary_alpha(image.data_mut(), threshold);

    match mode {
        FillMode::Fragment {
            iterations,
            fragment_count,
        } => fill_alpha_fragment_blur(image, iterations, fragment_count, temp),
        FillMode::ExtendColor { iterations } => fill_alpha_extend(image, iterations as usize),
        FillMode::Nearest {
            min_radius: radius,
            anti_aliasing,
        } => fill_alpha_nearest(image, radius, anti_aliasing),
    }
}

fn make_binary_alpha(pixels: &mut [Vec4], threshold: f32) {
    for p in pixels {
        let a: f32 = if p.w < threshold { 0. } else { 1. };
        *p *= a;
        p.w = a;
    }
}

fn fill_alpha_fragment_blur(
    image: &mut Image<Vec4>,
    iterations: u32,
    fragment_count: u32,
    temp: Option<&mut Image<Vec4>>,
) {
    if iterations == 0 {
        return;
    }

    let original = &*from_image_cow(image, temp);
    let mut buffer: Image<Vec4> = Image::from_const(image.size(), Vec4::ZERO);

    for i in 0..iterations {
        let radius = (1 << i) as f32;
        let angle_offset = i as f32;

        buffer = fragment_blur_alpha(
            original,
            radius,
            fragment_count as usize,
            angle_offset,
            Some(buffer),
        );
        overlay_self_mut(&mut buffer, 2);
        overlay_mut(&mut buffer, image);
        std::mem::swap(&mut buffer, image);
    }

    make_binary_alpha(image.data_mut(), 0.01);
}

fn is_to_fill(image: &Image<Vec4>, x: usize, y: usize) -> bool {
    let data = image.data();
    let w = image.width();
    let h = image.height();
    let i = y * w + x;

    data[i].w == 0.
        && (x > 0 && data[i - 1].w != 0.
            || x < w - 1 && data[i + 1].w != 0.
            || y > 0 && data[i - w].w != 0.
            || y < h - 1 && data[i + w].w != 0.)
}

fn get_fill(image: &Image<Vec4>, i: usize, x: usize, y: usize) -> Option<Vec4> {
    let w = image.width();
    let h = image.height();
    let data = image.data();

    if data[i].w == 0. {
        let mut acc = Vec4::ZERO;
        if x > 0 {
            acc += data[i - 1]
        }
        if x < w - 1 {
            acc += data[i + 1]
        }
        if y > 0 {
            acc += data[i - w]
        }
        if y < h - 1 {
            acc += data[i + w]
        }

        if acc.w != 0. {
            return Some(acc / acc.w);
        }
    }

    None
}
unsafe fn get_fill_unchecked(image: &Image<Vec4>, i: usize) -> Option<Vec4> {
    let w = image.width();
    let data = image.data();

    if data[i].w == 0. {
        let acc = *data.get_unchecked(i - 1)
            + *data.get_unchecked(i + 1)
            + *data.get_unchecked(i - w)
            + *data.get_unchecked(i + w);
        if acc.w != 0. {
            return Some(acc / acc.w);
        }
    }

    None
}
fn is_transparent(image: &Image<Vec4>, i: usize) -> bool {
    image.data()[i].w == 0.
}

fn fill_alpha_extend(image: &mut Image<Vec4>, iterations: usize) {
    if iterations == 0 {
        return;
    }

    let mut grid: Grid<8> = Grid::new(image.size());
    grid.fill_with_pixels(|x, y| is_to_fill(image, x, y));

    let mut fills = Vec::with_capacity(image.width().max(image.height()) * 4);

    for i in 0..iterations {
        if i > 0 && i % grid.cell_size() == 0 {
            grid.and_any(|x, y| is_to_fill(image, x, y));
        }
        if i % grid.cell_size() == 1 {
            grid.expand_one();
            grid.and_any_index(|i| is_transparent(image, i));
        }

        grid.for_each_true(|x_range, y_range, is_inner| {
            if is_inner {
                // inner cell
                for y in y_range {
                    for i in move_range(&x_range, y * image.width()) {
                        // SAFETY: This is an inner cell, so we are guaranteed to have at least one neighboring
                        // pixel in all directions.
                        if let Some(fill) = unsafe { get_fill_unchecked(image, i) } {
                            fills.push((i, fill));
                        }
                    }
                }
            } else {
                // border cell
                for y in y_range {
                    for x in x_range.clone() {
                        let i = y * image.width() + x;
                        if let Some(fill) = get_fill(image, i, x, y) {
                            fills.push((i, fill));
                        }
                    }
                }
            }
        });

        if fills.is_empty() {
            // no more filling is possible
            break;
        }

        let data = image.data_mut();
        for (i, fill) in fills.drain(..) {
            data[i] = fill;
        }
    }
}

fn within_radius_grid<const N: usize>(
    w: usize,
    h: usize,
    transparent: &[bool],
    radius: u32,
) -> Grid<N> {
    let size = Size::new(w, h);

    let mut transparency_grid = Grid::new(size);
    transparency_grid.fill_with_pixels_index(|i| transparent[i]);

    if radius as usize >= w || radius as usize >= h {
        return transparency_grid;
    }

    let mut opaque_grid = Grid::new(size);
    opaque_grid.fill_with_pixels_index(|i| !transparent[i]);

    let iters = div_ceil(radius as usize, opaque_grid.cell_size());
    for _ in 0..iters {
        // TODO: Make this less stupid
        opaque_grid.expand_one();
    }

    opaque_grid.and(&transparency_grid);
    opaque_grid
}

fn fill_alpha_nearest(image: &mut Image<Vec4>, radius: u32, anti_aliasing: bool) {
    let w = image.width();
    let h = image.height();
    let data = image.data_mut();

    let mut transparent = vec![false; w * h].into_boxed_slice();
    for (t, p) in transparent.iter_mut().zip(data.iter()) {
        *t = p.w == 0.;
    }

    let to_process: Grid<8> = within_radius_grid(w, h, &transparent, radius);

    // fill tree
    let mut points = Vec::with_capacity(w.max(h) * 4);
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if !transparent[i]
                && (x > 0 && transparent[i - 1]
                    || x < w - 1 && transparent[i + 1]
                    || y > 0 && transparent[i - w]
                    || y < h - 1 && transparent[i + w])
            {
                // opaque pixel surrounded by at least one transparent pixel
                points.push(GeomWithData::new((x as f32, y as f32), data[i]));
            }
        }
    }

    if points.is_empty() {
        return;
    }

    let tree = RTree::bulk_load(points);
    let mut samplers = Vec::from_iter(
        std::iter::repeat_with(|| None).take(to_process.width() * to_process.height()),
    );

    to_process.for_each_true_cell(|x_range, y_range, _, cell_index| {
        let (center, radius) = circle_around(&x_range, &y_range);
        let sampler = create_sampler_around(&tree, center, radius);
        samplers[cell_index] = Some(sampler);
    });

    // set pixels
    to_process.for_each_true_cell(|x_range, y_range, _, cell_index| {
        let sampler = samplers[cell_index].as_ref().unwrap();
        for y in y_range {
            for x in x_range.clone() {
                let i = y * w + x;
                if transparent[i] {
                    data[i] = sampler(x as f32, y as f32);
                }
            }
        }
    });

    if anti_aliasing {
        // find edges
        let mut edges = vec![false; w * h].into_boxed_slice();
        for y in 0..h {
            for i in move_range(&(1..w), y * w) {
                let p = data[i - 1];
                let n = data[i];
                if p != n {
                    edges[i - 1] = true;
                    edges[i] = true;
                }
            }
        }
        for x in 0..w {
            for y in 1..h {
                let i = y * w + x;
                let p = data[i - w];
                let n = data[i];
                if p != n {
                    edges[i - w] = true;
                    edges[i] = true;
                }
            }
        }

        // resolve edges
        to_process.for_each_true_cell(|x_range, y_range, _, cell_index| {
            let sampler = samplers[cell_index].as_ref().unwrap();

            for y in y_range {
                for x in x_range.clone() {
                    let i = y * w + x;
                    if transparent[i] && edges[i] {
                        let mut acc = data[i];

                        acc += sampler(x as f32 + 0.333, y as f32 + 0.333);
                        acc += sampler(x as f32 + 0.333, y as f32 - 0.333);
                        acc += sampler(x as f32 - 0.333, y as f32 + 0.333);
                        acc += sampler(x as f32 - 0.333, y as f32 - 0.333);
                        acc += sampler(x as f32, y as f32 + 0.333);
                        acc += sampler(x as f32, y as f32 - 0.333);
                        acc += sampler(x as f32 + 0.333, y as f32);
                        acc += sampler(x as f32 - 0.333, y as f32);

                        data[i] = acc / acc.w;
                    }
                }
            }
        });
    }
}

fn circle_around(x_range: &Range<usize>, y_range: &Range<usize>) -> ((f32, f32), f32) {
    let x_min = x_range.start as f32 - 0.5;
    let x_max = x_range.end as f32 - 0.5;
    let y_min = y_range.start as f32 - 0.5;
    let y_max = y_range.end as f32 - 0.5;

    (
        ((x_min + x_max) / 2., (y_min + y_max) / 2.),
        (x_max - x_min).max(y_max - y_min) + 1.,
    )
}

fn create_sampler_around(
    tree: &RTree<GeomWithData<(f32, f32), Vec4>>,
    center: (f32, f32),
    radius: f32,
) -> impl Fn(f32, f32) -> Vec4 + '_ {
    fn dist_sq(a: (f32, f32), b: (f32, f32)) -> f32 {
        let x = a.0 - b.0;
        let y = a.1 - b.1;
        x * x + y * y
    }

    let closest = tree.nearest_neighbor(&center).unwrap();
    let closest_dist = dist_sq(center, *closest.geom()).sqrt();
    let max_dist = closest_dist + radius * 2.;
    let max_dist_sq = max_dist * max_dist;

    let mut candidates: Vec<_> = tree.locate_within_distance(center, max_dist_sq).collect();
    candidates.sort_unstable_by_key(|a| {
        // we just need *a* key to sort elements by
        // as long as that key is unique for each element, we don't care what it is
        let (x, y) = *a.geom();
        (x.to_bits(), y.to_bits())
    });

    let candidates = candidates.into_boxed_slice();
    let first = candidates[0];

    move |x: f32, y: f32| {
        if candidates.len() == 1 {
            return first.data;
        }

        let mut min = first;
        let mut min_dist = dist_sq((x, y), *first.geom());

        for g in candidates[1..].iter() {
            let d = dist_sq((x, y), *g.geom());
            if d < min_dist {
                min_dist = d;
                min = g;
            }
        }

        min.data
    }
}

#[cfg(test)]
mod tests {
    use test_util::{data::read_flower_transparent, snap::ImageSnapshot};

    #[test]
    fn fill_alpha_texture() {
        let mut original = read_flower_transparent();
        super::fill_alpha(
            &mut original,
            0.15,
            super::FillMode::Fragment {
                iterations: 6,
                fragment_count: 5,
            },
            None,
        );
        original.snapshot("fill_alpha_texture");
    }

    #[test]
    fn fill_alpha_color() {
        let mut original = read_flower_transparent();
        super::fill_alpha(
            &mut original,
            0.15,
            super::FillMode::ExtendColor { iterations: 64 },
            None,
        );
        original.snapshot("fill_alpha_color");
    }

    #[test]
    fn fill_alpha_nearest() {
        let mut original = read_flower_transparent();
        super::fill_alpha(
            &mut original,
            0.15,
            super::FillMode::Nearest {
                min_radius: 50,
                anti_aliasing: false,
            },
            None,
        );
        original.snapshot("fill_alpha_nearest");
    }
}
