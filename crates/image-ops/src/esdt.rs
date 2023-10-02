// Source: https://gitlab.com/unconed/use.gpu/-/tree/master/packages/glyph
// MIT: Copyright (c) 2021-2022 Steven Wittens

use image_core::Image;

pub fn esdf(
    img: &Image<f32>,
    radius: f32,
    cutoff: f32,
    pre_process: bool,
    post_process: bool,
) -> Image<f32> {
    let w = img.width();
    let h = img.height();

    let mut stage = SDFStage::new(w, h);

    paint_into_stage(&mut stage, img);
    paint_subpixel_offsets(&mut stage, img, pre_process);

    esdt(
        &mut stage.outer,
        &mut stage.xo,
        &mut stage.yo,
        w,
        h,
        &mut stage.f,
        &mut stage.z,
        &mut stage.b,
        &mut stage.t,
        &mut stage.v,
    );
    esdt(
        &mut stage.inner,
        &mut stage.xi,
        &mut stage.yi,
        w,
        h,
        &mut stage.f,
        &mut stage.z,
        &mut stage.b,
        &mut stage.t,
        &mut stage.v,
    );
    if post_process {
        relax_subpixel_offsets(&mut stage, w, h);
    }

    let xo = &*stage.xo;
    let yo = &*stage.yo;
    let xi = &*stage.xi;
    let yi = &*stage.yi;
    let mut alpha = Image::new(
        img.size(),
        (0..img.len())
            .map(|i| {
                let outer = f32::max(0.0, f32::hypot(xo[i], yo[i]) - 0.5);
                let inner = f32::max(0.0, f32::hypot(xi[i], yi[i]) - 0.5);
                let d = if outer >= inner { outer } else { -inner };
                (1.0 - (d / radius + cutoff)).clamp(0.0, 1.0)
            })
            .collect(),
    );

    if !pre_process {
        paint_into_distance_field(alpha.data_mut(), img, radius, cutoff);
    }

    alpha
}

// Helpers
const INF: f32 = 1e10;

fn is_black(x: f32) -> bool {
    x <= 0.0
}
fn is_white(x: f32) -> bool {
    x >= 1.0
}
fn is_solid(x: f32) -> bool {
    is_white(x) || is_black(x)
}

struct SDFStage {
    outer: Box<[f32]>,
    inner: Box<[f32]>,

    xo: Box<[f32]>,
    yo: Box<[f32]>,
    xi: Box<[f32]>,
    yi: Box<[f32]>,

    f: Box<[f32]>,
    z: Box<[f32]>,
    b: Box<[f32]>,
    t: Box<[f32]>,
    v: Box<[usize]>,
}

impl SDFStage {
    pub fn new(w: usize, h: usize) -> Self {
        let size = w.max(h);
        let len = w * h;

        Self {
            outer: vec![INF; len].into_boxed_slice(),
            inner: vec![0.0; len].into_boxed_slice(),

            xo: vec![0.0; len].into_boxed_slice(),
            yo: vec![0.0; len].into_boxed_slice(),
            xi: vec![0.0; len].into_boxed_slice(),
            yi: vec![0.0; len].into_boxed_slice(),

            f: vec![0.0; size].into_boxed_slice(),
            z: vec![0.0; size + 1].into_boxed_slice(),
            b: vec![0.0; size].into_boxed_slice(),
            t: vec![0.0; size].into_boxed_slice(),
            v: vec![0; size].into_boxed_slice(),
        }
    }
}

// Paint alpha channel into SDF stage
fn paint_into_stage(stage: &mut SDFStage, img: &Image<f32>) {
    let outer = &mut *stage.outer;
    let inner = &mut *stage.inner;

    let data = img.data();

    for i in 0..data.len() {
        let a = data[i];
        if a == 0.0 {
            continue;
        }

        if is_white(a) {
            outer[i] = 0.0;
            inner[i] = INF;
        } else {
            outer[i] = 0.0;
            inner[i] = 0.0;
        }
    }
}

// Paint original alpha channel into final SDF when gray
fn paint_into_distance_field(output: &mut [f32], img: &Image<f32>, radius: f32, cutoff: f32) {
    assert_eq!(output.len(), img.len());
    output
        .iter_mut()
        .zip(img.data().iter())
        .for_each(|(o, &a)| {
            if !is_solid(a) {
                let d = 0.5 - a;
                *o = (1.0 - (d / radius + cutoff)).clamp(0.0, 1.0);
            }
        });
}

// Generate subpixel offsets for all border pixels
fn paint_subpixel_offsets(stage: &mut SDFStage, img: &Image<f32>, relax: bool) {
    let w = img.width();
    let h = img.height();

    let outer = &mut *stage.outer;
    let inner = &mut *stage.inner;

    let xo = &mut *stage.xo;
    let yo = &mut *stage.yo;
    let xi = &mut *stage.xi;
    let yi = &mut *stage.yi;

    let data = img.data();

    let get_data = |x: usize, y: usize| {
        debug_assert!(x < w);
        debug_assert!(y < h);
        data[y * w + x]
    };

    // Make vector from pixel center to nearest boundary
    for y in 0..h {
        let y_m1 = if y == 0 { 0 } else { y - 1 };
        let y_p1 = if y == h - 1 { h - 1 } else { y + 1 };
        for x in 0..w {
            let x_m1 = if x == 0 { 0 } else { x - 1 };
            let x_p1 = if x == w - 1 { w - 1 } else { x + 1 };

            let c = get_data(x, y);
            let j = y * w + x;

            if !is_solid(c) {
                let dc = c - 0.5;

                let l = get_data(x_m1, y);
                let r = get_data(x_p1, y);
                let t = get_data(x, y_m1);
                let b = get_data(x, y_p1);

                let tl = get_data(x_m1, y_m1);
                let tr = get_data(x_p1, y_m1);
                let bl = get_data(x_m1, y_p1);
                let br = get_data(x_p1, y_p1);

                let ll = (tl + l * 2.0 + bl) / 4.0;
                let rr = (tr + r * 2.0 + br) / 4.0;
                let tt = (tl + t * 2.0 + tr) / 4.0;
                let bb = (bl + b * 2.0 + br) / 4.0;

                fn min_many<const N: usize>(values: [f32; N]) -> f32 {
                    let mut min = values[0];
                    for i in values.into_iter().skip(1) {
                        min = min.min(i);
                    }
                    min
                }
                fn max_many<const N: usize>(values: [f32; N]) -> f32 {
                    let mut max = values[0];
                    for i in values.into_iter().skip(1) {
                        max = max.max(i);
                    }
                    max
                }
                let min = min_many([l, r, t, b, tl, tr, bl, br]);
                let max = max_many([l, r, t, b, tl, tr, bl, br]);

                if min > 0.0 {
                    // Interior creases
                    inner[j] = INF;
                    continue;
                }
                if max < 1.0 {
                    // Exterior creases
                    outer[j] = INF;
                    continue;
                }

                let mut dx = rr - ll;
                let mut dy = bb - tt;
                let dl = 1.0 / f32::hypot(dx, dy);
                dx *= dl;
                dy *= dl;

                xo[j] = -dc * dx;
                yo[j] = -dc * dy;
            } else if is_white(c) {
                let l = get_data(x_m1, y);
                let r = get_data(x_p1, y);
                let t = get_data(x, y_m1);
                let b = get_data(x, y_p1);

                if is_black(l) && x > 0 {
                    xo[j - 1] = 0.4999;
                    outer[j - 1] = 0.0;
                    inner[j - 1] = 0.0;
                }
                if is_black(r) && x < w - 1 {
                    xo[j + 1] = -0.4999;
                    outer[j + 1] = 0.0;
                    inner[j + 1] = 0.0;
                }

                if is_black(t) && y > 0 {
                    yo[j - w] = 0.4999;
                    outer[j - w] = 0.0;
                    inner[j - w] = 0.0;
                }
                if is_black(b) && y < h - 1 {
                    yo[j + w] = -0.4999;
                    outer[j + w] = 0.0;
                    inner[j + w] = 0.0;
                }
            }
        }
    }

    // Blend neighboring offsets but preserve normal direction
    // Uses xo as input, xi as output
    // Improves quality slightly, but slows things down.
    if relax {
        let check_cross = |nx: f32,
                           ny: f32,
                           dc: f32,
                           dl: f32,
                           dr: f32,
                           dxl: f32,
                           dyl: f32,
                           dxr: f32,
                           dyr: f32| {
            ((dxl * nx + dyl * ny) * (dc * dl) > 0.0)
                && ((dxr * nx + dyr * ny) * (dc * dr) > 0.0)
                && ((dxl * dxr + dyl * dyr) * (dl * dr) > 0.0)
        };

        for y in 0..h {
            let y_m1 = if y == 0 { 0 } else { y - 1 };
            let y_p1 = if y == h - 1 { h - 1 } else { y + 1 };
            for x in 0..w {
                let x_m1 = if x == 0 { 0 } else { x - 1 };
                let x_p1 = if x == w - 1 { w - 1 } else { x + 1 };

                let j = y * w + x;

                let nx = xo[j];
                let ny = yo[j];
                if nx == 0.0 && ny == 0.0 {
                    continue;
                }

                let c = get_data(x, y);
                let l = get_data(x_m1, y);
                let r = get_data(x_p1, y);
                let t = get_data(x, y_m1);
                let b = get_data(x, y_p1);

                let dxl = xo[y * w + x_m1];
                let dxr = xo[y * w + x_p1];
                let dxt = xo[y_m1 * w + x];
                let dxb = xo[y_p1 * w + x];

                let dyl = yo[y * w + x_m1];
                let dyr = yo[y * w + x_p1];
                let dyt = yo[y_m1 * w + x];
                let dyb = yo[y_p1 * w + x];

                let mut dx = nx;
                let mut dy = ny;
                let mut dw = 1;

                let dc = c - 0.5;
                let dl = l - 0.5;
                let dr = r - 0.5;
                let dt = t - 0.5;
                let db = b - 0.5;

                if !is_solid(l)
                    && !is_solid(r)
                    && check_cross(nx, ny, dc, dl, dr, dxl, dyl, dxr, dyr)
                {
                    dx += (dxl + dxr) / 2.0;
                    dy += (dyl + dyr) / 2.0;
                    dw += 1;
                }

                if !is_solid(t)
                    && !is_solid(b)
                    && check_cross(nx, ny, dc, dt, db, dxt, dyt, dxb, dyb)
                {
                    dx += (dxt + dxb) / 2.0;
                    dy += (dyt + dyb) / 2.0;
                    dw += 1;
                }

                if !is_solid(l)
                    && !is_solid(t)
                    && check_cross(nx, ny, dc, dl, dt, dxl, dyl, dxt, dyt)
                {
                    dx += (dxl + dxt - 1.0) / 2.0;
                    dy += (dyl + dyt - 1.0) / 2.0;
                    dw += 1;
                }

                if !is_solid(r)
                    && !is_solid(t)
                    && check_cross(nx, ny, dc, dr, dt, dxr, dyr, dxt, dyt)
                {
                    dx += (dxr + dxt + 1.0) / 2.0;
                    dy += (dyr + dyt - 1.0) / 2.0;
                    dw += 1;
                }

                if !is_solid(l)
                    && !is_solid(b)
                    && check_cross(nx, ny, dc, dl, db, dxl, dyl, dxb, dyb)
                {
                    dx += (dxl + dxb - 1.0) / 2.0;
                    dy += (dyl + dyb + 1.0) / 2.0;
                    dw += 1;
                }

                if !is_solid(r)
                    && !is_solid(b)
                    && check_cross(nx, ny, dc, dr, db, dxr, dyr, dxb, dyb)
                {
                    dx += (dxr + dxb + 1.0) / 2.0;
                    dy += (dyr + dyb + 1.0) / 2.0;
                    dw += 1;
                }

                let nn = f32::hypot(nx, ny);
                let ll = (dx * nx + dy * ny) / nn;

                dx = nx * ll / dw as f32 / nn;
                dy = ny * ll / dw as f32 / nn;

                xi[j] = dx;
                yi[j] = dy;
            }
        }
    }

    // Produce zero points for positive and negative DF, at +0.5 / -0.5.
    // Splits xs into xo/xi
    for y in 0..h {
        for x in 0..w {
            let j = y * w + x;

            let (nx, ny) = if relax {
                (xi[j], yi[j])
            } else {
                (xo[j], yo[j])
            };

            if nx == 0.0 && ny == 0.0 {
                continue;
            }

            let nn = f32::hypot(nx, ny);

            fn num_sign(x: f32) -> isize {
                if x > 0.0 {
                    1
                } else if x < 0.0 {
                    -1
                } else {
                    0
                }
            }
            let sx = if (nx / nn).abs() > 0.5 {
                num_sign(nx)
            } else {
                0
            };
            let sy = if (ny / nn).abs() > 0.5 {
                num_sign(ny)
            } else {
                0
            };

            let c = get_data(x, y);
            let d = get_data(
                (x as isize + sx).clamp(0, w as isize - 1) as usize,
                (y as isize + sy).clamp(0, h as isize - 1) as usize,
            );
            let s = num_sign(d - c);

            let mut dlo = nn + 0.4999 * s as f32;
            let mut dli = nn - 0.4999 * s as f32;

            dli /= nn;
            dlo /= nn;

            xo[j] = nx * dlo;
            yo[j] = ny * dlo;
            xi[j] = nx * dli;
            yi[j] = ny * dli;
        }
    }
}

// Snap distance targets to neighboring target points, if closer.
// Makes the SDF more correct and less XY vs YX dependent, but has only little effect on final contours.
fn relax_subpixel_offsets(stage: &mut SDFStage, w: usize, h: usize) {
    let xo = &mut *stage.xo;
    let yo = &mut *stage.yo;
    let xi = &mut *stage.xi;
    let yi = &mut *stage.yi;

    // Check if target's neighbor is closer
    let check =
        |xs: &mut [f32], ys: &mut [f32], x: isize, y: isize, dx: f32, dy: f32, d: f32, j: usize| {
            let x = x.clamp(0, w as isize - 1) as usize;
            let y = y.clamp(0, h as isize - 1) as usize;
            let k = y * w + x;

            let dx2 = dx + xs[k];
            let dy2 = dy + ys[k];
            let d2 = f32::hypot(dx2, dy2);

            if d2 < d {
                xs[j] = dx2;
                ys[j] = dy2;
                d2
            } else {
                d
            }
        };

    let relax = |xs: &mut [f32], ys: &mut [f32]| {
        for y in 0..h {
            for x in 0..w {
                let j = y * w + x;

                let dx = xs[j];
                let dy = ys[j];
                if dx == 0.0 && dy == 0.0 {
                    continue;
                }

                // Step towards target minus 0.5px to find contour
                let mut d = f32::hypot(dx, dy);
                let ds = (d - 0.5) / d;
                let tx = x as f32 + dx * ds;
                let ty = y as f32 + dy * ds;

                // Check area around array index
                let ix = tx.round() as isize;
                let iy = ty.round() as isize;

                let dx = ix as f32 - x as f32;
                let dy = iy as f32 - y as f32;
                d = check(xs, ys, ix + 1, iy, dx + 1.0, dy, d, j);
                d = check(xs, ys, ix - 1, iy, dx - 1.0, dy, d, j);
                d = check(xs, ys, ix, iy + 1, dx, dy + 1.0, d, j);
                check(xs, ys, ix, iy - 1, dx, dy - 1.0, d, j);
            }
        }
    };

    relax(xo, yo);
    relax(xi, yi);
}

// 2D subpixel distance transform by unconed
// extended from Felzenszwalb & Huttenlocher https://cs.brown.edu/~pff/papers/dt-final.pdf
#[allow(clippy::too_many_arguments)]
fn esdt(
    mask: &mut [f32],
    xs: &mut [f32],
    ys: &mut [f32],
    w: usize,
    h: usize,
    f: &mut [f32],
    z: &mut [f32],
    b: &mut [f32],
    t: &mut [f32],
    v: &mut [usize],
) {
    for x in 0..w {
        esdt1d(mask, ys, xs, x, w, h, f, z, b, t, v);
    }
    for y in 0..h {
        esdt1d(mask, xs, ys, y * w, 1, w, f, z, b, t, v);
    }
}

// 1D subpixel distance transform
#[allow(clippy::too_many_arguments)]
fn esdt1d(
    mask: &mut [f32],
    xs: &mut [f32],
    ys: &mut [f32],
    offset: usize,
    stride: usize,
    length: usize,
    f: &mut [f32],   // Squared distance
    z: &mut [f32],   // Voronoi threshold
    b: &mut [f32],   // Subpixel offset parallel
    t: &mut [f32],   // Subpixel offset perpendicular
    v: &mut [usize], // Array index
) {
    v[0] = 0;
    b[0] = xs[offset];
    t[0] = ys[offset];
    z[0] = -INF;
    z[1] = INF;
    f[0] = if mask[offset] != 0.0 {
        INF
    } else {
        ys[offset] * ys[offset]
    };

    // Scan along array and build list of critical minima
    {
        let mut k: isize = 0;
        for q in 1..length {
            let o = offset + q * stride;

            // Perpendicular
            let dx = xs[o];
            let dy = ys[o];
            let fq = if mask[o] != 0.0 { INF } else { dy * dy };
            f[q] = fq;
            t[q] = dy;

            // Parallel
            let qs = q as f32 + dx;
            let q2 = qs * qs;
            b[q] = qs;

            // Remove any minima eclipsed by this one
            let mut s;
            loop {
                assert!(k >= 0);
                let r = v[k as usize];
                let rs = b[r];

                let r2 = rs * rs;
                s = (fq - f[r] + q2 - r2) / (qs - rs) / 2.0;

                let cond = s <= z[k as usize] && {
                    k -= 1;
                    k
                } > -1;
                if !cond {
                    break;
                }
            }

            // Add to minima list
            k += 1;
            assert!(k >= 0);
            v[k as usize] = q;
            z[k as usize] = s;
            z[k as usize + 1] = INF;
        }
    }

    // Resample array based on critical minima
    {
        let mut k = 0;
        for q in 0..length {
            // Skip eclipsed minima
            while z[k + 1] < q as f32 {
                k += 1;
            }

            let r = v[k];
            let rs = b[r];
            let dy = t[r];

            // Distance from integer index to subpixel location of minimum
            let rq = rs - q as f32;

            let o = offset + q * stride;
            xs[o] = rq;
            ys[o] = dy;

            // Mark cell as having propagated
            if r != q {
                mask[o] = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use test_util::{
        data::{read_at, read_binary_alpha, read_checker, read_flower},
        snap::ImageSnapshot,
    };

    #[test]
    fn at() {
        let original = read_at();
        super::esdf(&original, 200.0, 0.25, false, false).snapshot("at_esdf");
    }

    #[test]
    fn flower() {
        let original = read_flower().map(|p| p.x);
        super::esdf(&original, 10.0, 0.25, false, false).snapshot("flower_esdf");
    }

    #[test]
    fn checker() {
        let original = read_checker();
        super::esdf(&original, 10.0, 0.5, false, false).snapshot("checker_esdf");
    }

    #[test]
    fn binary_alpha() {
        let original = read_binary_alpha();
        super::esdf(&original, 20.0, 0.5, false, false).snapshot("binary_alpha_esdf");
    }
}
