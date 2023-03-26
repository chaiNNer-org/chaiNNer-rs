use glam::Vec4;
use image_core::{Image, Size};
use std::{ops::Range, slice::ChunksMut};

use crate::{
    blend::{overlay_mut, overlay_self_mut},
    fragment_blur::fragment_blur_alpha,
    util::{from_image_cow, move_range, process_pairs},
};

pub enum FillMode {
    Texture {
        iterations: u32,
        fragment_count: u32,
    },
    Color {
        iterations: u32,
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
        FillMode::Texture {
            iterations,
            fragment_count,
        } => fill_alpha_fragment_blur(image, iterations, fragment_count, temp),
        FillMode::Color { iterations } => fill_alpha_extend(image, iterations as usize),
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

struct Grid {
    cells: Box<[bool]>,
    width: usize,
    height: usize,
}

impl Grid {
    const CELL_SIZE: usize = 8;

    fn get_grid_size(size: Size) -> (usize, usize) {
        let grid_w = (size.width as f64 / Self::CELL_SIZE as f64).ceil() as usize;
        let grid_h = (size.height as f64 / Self::CELL_SIZE as f64).ceil() as usize;
        (grid_w, grid_h)
    }
    fn cell_to_pixel(
        (cell_x, cell_y): (usize, usize),
        image_size: Size,
    ) -> (Range<usize>, Range<usize>) {
        (
            (cell_x * Self::CELL_SIZE)..(((cell_x + 1) * Self::CELL_SIZE).min(image_size.width)),
            (cell_y * Self::CELL_SIZE)..(((cell_y + 1) * Self::CELL_SIZE).min(image_size.height)),
        )
    }

    fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![false; width * height].into_boxed_slice(),
            width,
            height,
        }
    }

    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }

    fn get(&self, x: usize, y: usize) -> bool {
        self.get_index(y * self.width() + x)
    }
    #[allow(unused)]
    fn set(&mut self, x: usize, y: usize, value: bool) {
        self.set_index(y * self.width() + x, value);
    }
    fn get_index(&self, i: usize) -> bool {
        self.cells[i]
    }
    fn set_index(&mut self, i: usize, value: bool) {
        self.cells[i] = value;
    }

    fn from_image_size(size: Size) -> Self {
        let (w, h) = Self::get_grid_size(size);
        Self::new(w, h)
    }

    fn fill_with_image<P>(&mut self, image: &Image<P>, f: impl Fn(usize, usize) -> bool) {
        assert!((self.width(), self.height()) == Self::get_grid_size(image.size()));

        self.fill_cells(|_, cx, cy| {
            let (x_range, y_range) = Self::cell_to_pixel((cx, cy), image.size());
            for y in y_range {
                for x in x_range.clone() {
                    if f(x, y) {
                        return true;
                    }
                }
            }
            false
        })
    }
    fn and_any<P>(&mut self, image: &Image<P>, f: impl Fn(usize, usize) -> bool) {
        assert!((self.width(), self.height()) == Self::get_grid_size(image.size()));

        self.fill_cells(|old, cx, cy| {
            if !old {
                return false;
            }
            let (x_range, y_range) = Self::cell_to_pixel((cx, cy), image.size());
            for y in y_range {
                for x in x_range.clone() {
                    if f(x, y) {
                        return true;
                    }
                }
            }
            false
        })
    }
    fn fill_cells(&mut self, f: impl Fn(bool, usize, usize) -> bool) {
        let w = self.width();
        let h = self.height();

        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                self.set_index(i, f(self.get_index(i), x, y));
            }
        }
    }

    fn expand_one(&mut self) {
        fn get_lines(s: &mut Grid) -> ChunksMut<'_, bool> {
            let w = s.width();
            s.cells.chunks_mut(w)
        }
        fn or_many(a: &mut [bool], b: &mut [bool]) {
            for (a, b) in a.iter_mut().zip(b) {
                *a = *a || *b;
            }
        }
        fn or(a: &mut bool, b: &mut bool) {
            *a = *a || *b;
        }

        // expand along y
        process_pairs(get_lines(self), or_many);
        process_pairs(get_lines(self).rev(), or_many);

        // expand along x
        for line in get_lines(self) {
            process_pairs(line.iter_mut(), or);
            process_pairs(line.iter_mut().rev(), or);
        }
    }
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

    let mut grid = Grid::from_image_size(image.size());
    grid.fill_with_image(image, |x, y| is_to_fill(image, x, y));

    let mut fills = Vec::with_capacity(image.width().max(image.height()) * 4);

    for i in 0..iterations {
        if i > 0 && i % Grid::CELL_SIZE == 0 {
            grid.and_any(image, |x, y| is_to_fill(image, x, y));
        }
        if i % Grid::CELL_SIZE == 1 {
            grid.expand_one();
            grid.and_any(image, |x, y| is_transparent(image, y * image.width() + x));
        }

        let inner_x = 1..(grid.width() - 1);
        let inner_y = 1..(grid.height() - 1);

        for cell_y in 0..grid.height() {
            for cell_x in 0..grid.width() {
                if grid.get(cell_x, cell_y) {
                    let (x_range, y_range) = Grid::cell_to_pixel((cell_x, cell_y), image.size());

                    if inner_x.contains(&cell_x) && inner_y.contains(&cell_y) {
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
                }
            }
        }

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

#[cfg(test)]
mod tests {
    use test_util::{data::read_flower_transparent, snap::ImageSnapshot};

    #[test]
    fn fill_alpha_texture() {
        let mut original = read_flower_transparent();
        super::fill_alpha(
            &mut original,
            0.15,
            super::FillMode::Texture {
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
            super::FillMode::Color { iterations: 64 },
            None,
        );
        original.snapshot("fill_alpha_color");
    }
}
