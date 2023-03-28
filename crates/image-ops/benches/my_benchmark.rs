use criterion::{black_box, criterion_group, criterion_main, Criterion};
use image_ops::{
    fill_alpha::{fill_alpha, FillMode},
    fragment_blur::{fragment_blur, fragment_blur_alpha},
};
use test_util::data::{read_flower, read_flower_transparent};

fn criterion_benchmark(c: &mut Criterion) {
    let img = black_box(read_flower());
    let img_t = black_box(read_flower_transparent());

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
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
