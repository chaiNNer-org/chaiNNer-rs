use image_core::Image;

use super::util::{write_2x, write_3x};

pub fn adv_mame_2x<T>(src: &Image<T>) -> Image<T>
where
    T: Default + Copy + PartialEq,
{
    // implemented using only the documentation here:
    // https://en.wikipedia.org/w/index.php?title=Pixel-art_scaling_algorithms&oldid=1181447123#EPX/Scale2%C3%97/AdvMAME2%C3%97

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

            //   A
            // C P B
            //   D
            let p = src[y * w + x];
            let a = src[y_m1 * w + x];
            let b = src[y * w + x_p1];
            let c = src[y * w + x_m1];
            let d = src[y_p1 * w + x];

            let mut r1 = p;
            let mut r2 = p;
            let mut r3 = p;
            let mut r4 = p;

            let eq_ab = a == b;
            let eq_ac = a == c;
            let eq_bd = b == d;
            let eq_cd = c == d;

            if eq_ac && !eq_cd && !eq_ab {
                r1 = a;
            }
            if eq_ab && !eq_ac && !eq_bd {
                r2 = b;
            }
            if eq_cd && !eq_ac && !eq_bd {
                r3 = c;
            }
            if eq_bd && !eq_ab && !eq_cd {
                r4 = d;
            }

            write_2x(dest, w, x, y, [r1, r2, r3, r4]);
        }
    }

    result
}

pub fn adv_mame_3x<T>(src: &Image<T>) -> Image<T>
where
    T: Default + Copy + PartialEq,
{
    // implemented using only the documentation here:
    // https://en.wikipedia.org/w/index.php?title=Pixel-art_scaling_algorithms&oldid=1181447123#EPX/Scale2%C3%97/AdvMAME2%C3%97

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

            if b == d && d != h && b != f {
                r1 = d;
            }
            if (b == d && d != h && b != f && c != e) || (b == f && b != d && f != h && a != e) {
                r2 = b;
            }
            if b == f && b != d && f != h {
                r3 = f;
            }
            if (d == h && f != h && b != d && a != e) || (b == d && d != h && b != f && e != g) {
                r4 = d;
            }
            if (b == f && b != d && f != h && e != i) || (f == h && b != f && d != h && c != e) {
                r6 = f;
            }
            if d == h && f != h && b != d {
                r7 = d;
            }
            if (f == h && b != f && d != h && e != g) || (d == h && f != h && b != d && e != i) {
                r8 = h;
            }
            if f == h && b != f && d != h {
                r9 = f;
            }

            write_3x(dest, w, x, y, [r1, r2, r3, r4, r5, r6, r7, r8, r9]);
        }
    }

    result
}

pub fn adv_mame_4x<T>(src: &Image<T>) -> Image<T>
where
    T: Default + Copy + PartialEq,
{
    // implemented using only the documentation here:
    // https://en.wikipedia.org/w/index.php?title=Pixel-art_scaling_algorithms&oldid=1181447123#EPX/Scale2%C3%97/AdvMAME2%C3%97

    adv_mame_2x(&adv_mame_2x(src))
}

#[cfg(test)]
mod tests {
    use test_util::{data::read_nes_smb, snap::ImageSnapshot};

    #[test]
    fn adv_mame() {
        let original = read_nes_smb();

        super::adv_mame_2x(&original).snapshot("px_up_adv_mame_2x");
        super::adv_mame_3x(&original).snapshot("px_up_adv_mame_3x");
        super::adv_mame_4x(&original).snapshot("px_up_adv_mame_4x");
    }
}
