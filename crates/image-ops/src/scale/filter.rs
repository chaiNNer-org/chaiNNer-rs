#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Filter {
    Nearest,
    Box,
    Linear,
    Hermite,
    CubicCatrom,
    CubicMitchell,
    CubicBSpline,
    Hamming,
    Hann,
    Lanczos3,
    Lagrange,
    Gauss,
}

#[inline]
fn sinc(x: f32) -> f32 {
    if x == 0.0 {
        1.0
    } else {
        x.sin() / x
    }
}

// Taken from
// https://github.com/PistonDevelopers/image/blob/2921cd7/src/imageops/sample.rs#L68
// TODO(Kagami): Could be optimized for known B and C, see e.g.
// https://github.com/sekrit-twc/zimg/blob/1a606c0/src/zimg/resize/filter.cpp#L149
#[inline(always)]
fn cubic_bc(b: f32, c: f32, x: f32) -> f32 {
    let a = x.abs();
    let k = if a < 1.0 {
        (12.0 - 9.0 * b - 6.0 * c) * a.powi(3)
            + (-18.0 + 12.0 * b + 6.0 * c) * a.powi(2)
            + (6.0 - 2.0 * b)
    } else if a < 2.0 {
        (-b - 6.0 * c) * a.powi(3)
            + (6.0 * b + 30.0 * c) * a.powi(2)
            + (-12.0 * b - 48.0 * c) * a
            + (8.0 * b + 24.0 * c)
    } else {
        0.0
    };
    k / 6.0
}

fn lagrange(x: f32, support: f32) -> f32 {
    let x = x.abs();
    if x > support {
        return 0.0;
    }

    // Taken from
    // https://github.com/ImageMagick/ImageMagick/blob/e8b7974e8756fb278ec85d896065a1b96ed85af9/MagickCore/resize.c#L406
    let order = (2.0 * support) as isize;
    let n = (support + x) as isize;
    let mut value = 1.0;
    for i in 0..order {
        let d = (n - i) as f32;
        if d != 0.0 {
            value *= (d - x) / d;
        }
    }
    value
}

impl From<Filter> for resize::Type {
    fn from(filter: Filter) -> Self {
        match filter {
            Filter::Nearest => resize::Type::Point,
            Filter::Box => {
                let filter =
                    resize::Filter::new(Box::new(|x| if x.abs() <= 0.5 { 1.0 } else { 0.0 }), 1.0);
                resize::Type::Custom(filter)
            }
            Filter::Linear => resize::Type::Triangle,
            Filter::Hermite => {
                let filter = resize::Filter::new(Box::new(|x| cubic_bc(0.0, 0.0, x)), 1.0);
                resize::Type::Custom(filter)
            }
            Filter::CubicCatrom => resize::Type::Catrom,
            Filter::CubicMitchell => resize::Type::Mitchell,
            Filter::CubicBSpline => resize::Type::BSpline,
            Filter::Hamming => {
                let filter = resize::Filter::new(
                    Box::new(|x| {
                        let x = x.abs() * std::f32::consts::PI;
                        sinc(x) * (0.54 + 0.46 * x.cos())
                    }),
                    1.0,
                );
                resize::Type::Custom(filter)
            }
            Filter::Hann => {
                let filter = resize::Filter::new(
                    Box::new(|x| {
                        let x = x.abs() * std::f32::consts::PI;
                        sinc(x) * (0.5 + 0.5 * x.cos())
                    }),
                    1.0,
                );
                resize::Type::Custom(filter)
            }
            Filter::Lanczos3 => resize::Type::Lanczos3,
            Filter::Lagrange => {
                let filter = resize::Filter::new(Box::new(|x| lagrange(x, 2.0)), 2.0);
                resize::Type::Custom(filter)
            }
            Filter::Gauss => resize::Type::Gaussian,
        }
    }
}
