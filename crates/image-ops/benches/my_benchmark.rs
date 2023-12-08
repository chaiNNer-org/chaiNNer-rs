#![allow(unused)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use image_core::{NDimImage, Shape};
use image_ops::{
    dither::*,
    esdt::esdf,
    fill_alpha::{fill_alpha, FillMode},
    fragment_blur::{fragment_blur, fragment_blur_alpha},
    palette::extract_unique_ndim,
    threshold::binary_threshold,
};
use test_util::data::{
    read_at, read_flower, read_flower_palette, read_flower_transparent, read_lion,
};

fn criterion_benchmark(c: &mut Criterion) {
    let img = black_box(read_flower());
    let img_t = black_box(read_flower_transparent());
    let img_lion = black_box(read_lion());
    let img_lion_ndim: NDimImage = black_box(read_lion()).into();
    let img_lion_red = NDimImage::new(
        Shape::from_size(img_lion.size(), 1),
        img_lion.data().iter().map(|p| p.x).collect(),
    );
    let img_at = read_at();

    c.bench_function("fragment rgb r=20 c=5", |b| {
        b.iter(|| fragment_blur(&img, 20., 5, 0., None))
    });
    c.bench_function("fragment rgb r=20 c=10", |b| {
        b.iter(|| fragment_blur(&img, 20., 10, 0., None))
    });
    c.bench_function("fragment rgba r=20 c=10", |b| {
        b.iter(|| fragment_blur_alpha(&img_t, 20., 10, 0., None))
    });

    c.bench_function("fill alpha texture", |b| {
        b.iter(|| {
            let mut i = img_t.clone();
            fill_alpha(
                &mut i,
                0.15,
                FillMode::Fragment {
                    iterations: 8,
                    fragment_count: 5,
                },
                None,
            )
        })
    });

    c.bench_function("fill alpha color", |b| {
        b.iter(|| {
            let mut i = img_t.clone();
            fill_alpha(
                &mut i,
                0.15,
                FillMode::ExtendColor { iterations: 1000 },
                None,
            )
        })
    });

    c.bench_function("fill alpha nearest", |b| {
        b.iter(|| {
            let mut i = img_t.clone();
            fill_alpha(
                &mut i,
                0.15,
                FillMode::Nearest {
                    min_radius: u32::MAX,
                    anti_aliasing: false,
                },
                None,
            )
        })
    });

    c.bench_function("distinct colors grayscale", |b| {
        b.iter(|| extract_unique_ndim(img_lion_red.view(), usize::MAX))
    });
    c.bench_function("distinct colors rgb", |b| {
        b.iter(|| extract_unique_ndim(img_lion_ndim.view(), usize::MAX))
    });
    c.bench_function("error diffusion dither map", |b| {
        b.iter(|| {
            error_diffusion_dither_map(&img, FloydSteinberg, &ChannelQuantization::new(4), None);
        })
    });
    c.bench_function("error diffusion dither", |b| {
        let mut img = img.clone();
        b.iter(|| {
            error_diffusion_dither(&mut img, FloydSteinberg, &ChannelQuantization::new(4));
        })
    });
    c.bench_function("riemersma dither", |b| {
        let mut img = img.clone();
        b.iter(|| {
            riemersma_dither(&mut img, 16, 1.0 / 16.0, &ChannelQuantization::new(4));
        })
    });
    c.bench_function("ordered dither", |b| {
        let mut flower_nd: NDimImage = img.clone().into();
        b.iter(|| {
            ordered_dither(&mut flower_nd, 4, ChannelQuantization::new(2));
        })
    });
    c.bench_function("quantize", |b| {
        let mut flower_nd: NDimImage = img.clone().into();
        b.iter(|| {
            quantize_ndim(&mut flower_nd, ChannelQuantization::new(4));
        })
    });
    c.bench_function("error diffusion dither palette", |b| {
        let mut img = img.clone();
        let palette = black_box(read_flower_palette());
        let quant = ColorPalette::new(RGB, palette.row(0).iter().copied(), BoundError);
        b.iter(|| {
            error_diffusion_dither(&mut img, FloydSteinberg, &quant);
        })
    });

    // c.bench_function("threshold", |b| {
    //     let img = img_lion_ndim.view();
    //     b.iter(|| {
    //         binary_threshold(img, 0.5, true);
    //     })
    // });

    c.bench_function("esdt", |b| {
        b.iter(|| {
            esdf(&img_at, 200.0, 0.25, false, false);
        })
    });

    c.bench_function("gamma", |b| {
        b.iter(|| {
            let mut img = img_lion_ndim.clone();
            image_ops::gamma::gamma_ndim(&mut img, 2.2);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
