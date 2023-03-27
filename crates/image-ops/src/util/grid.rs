use std::ops::Range;

use image_core::Size;

use crate::util::process_pairs;

use super::{div_ceil, move_range, FixedBits};

#[derive(Debug, Clone)]
pub struct Grid<const CELL_SIZE: usize = 8> {
    lines: Box<[FixedBits]>,
    width: usize,
    pixels: Size,
}

impl<const CELL_SIZE: usize> Grid<CELL_SIZE> {
    pub const fn cell_size(&self) -> usize {
        CELL_SIZE
    }

    fn cell_to_pixel_dimension(cell: usize, image_size: usize) -> Range<usize> {
        let start = cell * CELL_SIZE;
        let end = ((cell + 1) * CELL_SIZE).min(image_size);
        start..end
    }
    fn cell_to_pixel(
        (cell_x, cell_y): (usize, usize),
        image_size: Size,
    ) -> (Range<usize>, Range<usize>) {
        (
            Self::cell_to_pixel_dimension(cell_x, image_size.width),
            Self::cell_to_pixel_dimension(cell_y, image_size.height),
        )
    }

    pub fn new(pixels: Size) -> Self {
        let width = div_ceil(pixels.width, CELL_SIZE);
        let height = div_ceil(pixels.height, CELL_SIZE);

        Self {
            lines: vec![FixedBits::new(width); height].into_boxed_slice(),
            width,
            pixels,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.lines.len()
    }
    pub fn is_empty(&self) -> bool {
        self.width() == 0 || self.height() == 0
    }
    pub fn pixels(&self) -> Size {
        self.pixels
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        self.lines[y].get(x).unwrap()
    }
    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        self.lines[y].set(x, value)
    }

    pub fn fill_with_pixels(&mut self, f: impl Fn(usize, usize) -> bool) {
        let size = self.pixels();
        self.fill_cells(|_, cx, cy| {
            let (x_range, y_range) = Self::cell_to_pixel((cx, cy), size);
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
    pub fn fill_with_pixels_index(&mut self, f: impl Fn(usize) -> bool) {
        let size = self.pixels();
        self.fill_cells(|_, cx, cy| {
            let (x_range, y_range) = Self::cell_to_pixel((cx, cy), size);
            for y in y_range {
                for i in move_range(&x_range, y * size.width) {
                    if f(i) {
                        return true;
                    }
                }
            }
            false
        })
    }

    pub fn and_any(&mut self, f: impl Fn(usize, usize) -> bool) {
        let size = self.pixels();
        self.fill_cells(|old, cx, cy| {
            if !old {
                return false;
            }
            let (x_range, y_range) = Self::cell_to_pixel((cx, cy), size);
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
    pub fn and_any_index(&mut self, f: impl Fn(usize) -> bool) {
        let size = self.pixels();
        self.fill_cells(|old, cx, cy| {
            if !old {
                return false;
            }
            let (x_range, y_range) = Self::cell_to_pixel((cx, cy), size);
            for y in y_range {
                for i in move_range(&x_range, y * size.width) {
                    if f(i) {
                        return true;
                    }
                }
            }
            false
        })
    }

    pub fn fill_cells(&mut self, f: impl Fn(bool, usize, usize) -> bool) {
        let w = self.width();
        let h = self.height();

        for y in 0..h {
            for x in 0..w {
                self.set(x, y, f(self.get(x, y), x, y));
            }
        }
    }

    pub fn and(&mut self, other: &Self) {
        assert_eq!(self.width(), other.width());
        assert_eq!(self.height(), other.height());

        for (a, b) in self.lines.iter_mut().zip(other.lines.iter()) {
            a.and(b)
        }
    }

    pub fn expand_one(&mut self) {
        fn or_many(a: &mut FixedBits, b: &mut FixedBits) {
            a.or(b)
        }

        // expand along y
        process_pairs(self.lines.iter_mut(), or_many);
        process_pairs(self.lines.iter_mut().rev(), or_many);

        // expand along x
        for line in self.lines.iter_mut() {
            line.expand_one()
        }
    }

    pub fn for_each_true(&self, mut f: impl FnMut(Range<usize>, Range<usize>, bool)) {
        if self.is_empty() {
            return;
        }

        if self.width() == 1 {
            let size = self.pixels();
            let x_range = Self::cell_to_pixel_dimension(0, size.width);
            for y in 0..self.height() {
                let y_range = Self::cell_to_pixel_dimension(y, size.height);
                if self.get(0, y) {
                    f(x_range.clone(), y_range, false);
                }
            }
        } else {
            let inner_y = 1..(self.height() - 1);
            let size = self.pixels();

            let x_first = 0;
            let x_last = self.width() - 1;

            for y in 0..self.height() {
                let y_range = Self::cell_to_pixel_dimension(y, size.height);
                let is_inner_y = inner_y.contains(&y);

                if self.get(x_first, y) {
                    let x_range = Self::cell_to_pixel_dimension(x_first, size.width);
                    f(x_range, y_range.clone(), false);
                }

                for x in 1..(self.width() - 1) {
                    if self.get(x, y) {
                        let x_range = Self::cell_to_pixel_dimension(x, size.width);
                        f(x_range, y_range.clone(), is_inner_y);
                    }
                }

                if self.get(x_last, y) {
                    let x_range = Self::cell_to_pixel_dimension(x_last, size.width);
                    f(x_range, y_range.clone(), false);
                }
            }
        }
    }
    pub fn for_each_true_cell(&self, mut f: impl FnMut(Range<usize>, Range<usize>, bool, usize)) {
        if self.is_empty() {
            return;
        }

        let inner_x = 1..(self.width() - 1);
        let inner_y = 1..(self.height() - 1);
        let size = self.pixels();

        for y in 0..self.height() {
            let y_range = Self::cell_to_pixel_dimension(y, size.height);

            for x in 0..self.width() {
                if self.get(x, y) {
                    let x_range = Self::cell_to_pixel_dimension(x, size.width);

                    let is_inner = inner_x.contains(&x) && inner_y.contains(&y);
                    let cell_index = y * self.width() + x;
                    f(x_range, y_range.clone(), is_inner, cell_index);
                }
            }
        }
    }
}
