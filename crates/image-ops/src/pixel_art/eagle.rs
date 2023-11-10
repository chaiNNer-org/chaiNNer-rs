use image_core::Image;

use super::util::{write_2x, write_3x};

pub fn eagle_2x<T>(src: &Image<T>) -> Image<T>
where
    T: Default + Copy + PartialEq,
{
    // implemented using only the documentation here:
    // https://en.wikipedia.org/w/index.php?title=Pixel-art_scaling_algorithms&oldid=1181447123#Eagle

    let mut result = Image::from_const(src.size().scale(2.0), T::default());

    let w = src.width();
    let h = src.height();
    let src = src.data();

    let dest = result.data_mut();

    for y in 0..h {
        let y_m1 = y.saturating_sub(1);
        let y_p1 = (y + 1).min(h - 1);

        for x in 0..w {
            let x_m1 = x.saturating_sub(1);
            let x_p1 = (x + 1).min(w - 1);

            // A B C
            // D E F
            // G H I
            let a = src[y_m1 * w + x_m1];
            let b = src[y_m1 * w + x];
            let c = src[y_m1 * w + x_p1];
            let d = src[y * w + x_m1];
            let e = src[y * w + x];
            let f = src[y * w + x_p1];
            let g = src[y_p1 * w + x_m1];
            let h = src[y_p1 * w + x];
            let i = src[y_p1 * w + x_p1];

            let mut r1 = e;
            let mut r2 = e;
            let mut r3 = e;
            let mut r4 = e;

            if d == a && a == b {
                r1 = a;
            }
            if b == c && c == f {
                r2 = c;
            }
            if h == g && g == d {
                r3 = g;
            }
            if f == i && i == h {
                r4 = i;
            }

            write_2x(dest, w, x, y, [r1, r2, r3, r4]);
        }
    }

    result
}

pub fn eagle_3x<T>(src: &Image<T>) -> Image<T>
where
    T: Default + Copy + PartialEq,
{
    // implemented using only the documentation here:
    // https://en.wikipedia.org/w/index.php?title=Pixel-art_scaling_algorithms&oldid=1181447123#Eagle
    //
    // I applied a natural extension from 2x to 3x.

    let mut result = Image::from_const(src.size().scale(3.0), T::default());

    let w = src.width();
    let h = src.height();
    let src = src.data();

    let dest = result.data_mut();

    for y in 0..h {
        let y_m1 = y.saturating_sub(1);
        let y_p1 = (y + 1).min(h - 1);

        for x in 0..w {
            let x_m1 = x.saturating_sub(1);
            let x_p1 = (x + 1).min(w - 1);

            // A B C
            // D E F
            // G H I
            let a = src[y_m1 * w + x_m1];
            let b = src[y_m1 * w + x];
            let c = src[y_m1 * w + x_p1];
            let d = src[y * w + x_m1];
            let e = src[y * w + x];
            let f = src[y * w + x_p1];
            let g = src[y_p1 * w + x_m1];
            let h = src[y_p1 * w + x];
            let i = src[y_p1 * w + x_p1];

            let mut r1 = e;
            let mut r2 = e;
            let mut r3 = e;
            let mut r4 = e;
            let r5 = e;
            let mut r6 = e;
            let mut r7 = e;
            let mut r8 = e;
            let mut r9 = e;

            if d == a && a == b {
                r1 = a;
            }
            if a == b && b == c && (d == b || b == f) {
                r2 = b;
            }
            if b == c && c == f {
                r3 = c;
            }
            if a == d && d == g && (b == d || d == h) {
                r4 = d;
            }
            if c == f && f == i && (b == f || f == h) {
                r6 = f;
            }
            if h == g && g == d {
                r7 = g;
            }
            if g == h && h == i && (d == h || h == f) {
                r8 = h;
            }
            if f == i && i == h {
                r9 = i;
            }

            write_3x(dest, w, x, y, [r1, r2, r3, r4, r5, r6, r7, r8, r9]);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use test_util::{data::read_nes_smb, snap::ImageSnapshot};

    #[test]
    fn eagle() {
        let original = read_nes_smb();

        super::eagle_2x(&original).snapshot("px_up_eagle_2x");
        super::eagle_3x(&original).snapshot("px_up_eagle_3x");
    }
}
