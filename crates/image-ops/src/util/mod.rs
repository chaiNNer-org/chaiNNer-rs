mod bilinear;
mod bits;
mod grid;
mod image;

use std::ops::Range;

pub use bilinear::*;
pub use bits::FixedBits;
pub use grid::Grid;
pub use image::*;

#[inline(always)]
pub const fn div_ceil(a: usize, b: usize) -> usize {
    a / b + ((a % b != 0) as usize)
}

pub const fn move_range_i(range: &Range<usize>, offset: isize) -> Range<usize> {
    Range {
        start: (range.start as isize + offset) as usize,
        end: (range.end as isize + offset) as usize,
    }
}
pub const fn move_range(range: &Range<usize>, offset: usize) -> Range<usize> {
    Range {
        start: range.start + offset,
        end: range.end + offset,
    }
}

pub fn process_pairs<'a, T: 'a + ?Sized>(
    iter: impl IntoIterator<Item = &'a mut T>,
    mut f: impl FnMut(&mut T, &mut T),
) {
    let mut iter = iter.into_iter();
    if let Some(mut prev) = iter.next() {
        for next in iter {
            f(prev, next);
            prev = next;
        }
    }
}
