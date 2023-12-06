use image_core::NDimImage;
use rayon::prelude::*;

pub fn gamma_ndim(image: &mut NDimImage, gamma: f32) {
    // we want to divide the image into chunks
    const BLOCK_SIZE: usize = 1024 * 8;

    if image.channels() == 4 {
        // only apply gamma to RGB channels
        image
            .data_mut()
            .par_chunks_mut(BLOCK_SIZE)
            .for_each(|chunk| {
                // the AVX implementation is actually slower than the trivial one,
                // so we don't use it here

                let (chunks, rest) = image_core::util::slice_as_chunks_mut::<f32, 4>(chunk);
                assert!(rest.is_empty());

                chunks.iter_mut().for_each(|p| {
                    // only apply gamma to the RGB channels
                    p[0] = p[0].powf(gamma);
                    p[1] = p[1].powf(gamma);
                    p[2] = p[2].powf(gamma);
                });
            });
    } else {
        image
            .data_mut()
            .par_chunks_mut(BLOCK_SIZE)
            .for_each(|chunk| {
                // we want to use AVX2 if possible, because it's ~2x faster
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                {
                    if is_x86_feature_detected!("avx2") {
                        let (chunks, rest) = image_core::util::slice_as_chunks_mut::<f32, 8>(chunk);

                        // do the rest first
                        rest.iter_mut().for_each(|f| *f = f.powf(gamma));

                        chunks
                            .iter_mut()
                            .for_each(|f| unsafe { avx2::pow_clamp(f, gamma) });
                        return;
                    }
                }

                // fallback
                chunk.iter_mut().for_each(|f| *f = f.powf(gamma));
            });
    }
}

#[allow(clippy::excessive_precision)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {

    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    #[target_feature(enable = "avx2")]
    pub unsafe fn pow_clamp(x: &mut [f32; 8], y: f32) {
        // pow(x, y) == exp(y * log(x))
        let x_m = _mm256_loadu_ps(x as *const _);
        let y_m = _mm256_set1_ps(y);
        let p = pow(x_m, y_m);

        // clamp to 0..1
        let p = _mm256_max_ps(p, _mm256_setzero_ps());
        let p = _mm256_min_ps(p, ONE);

        _mm256_storeu_ps(x as *mut _, p);
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn pow(x: __m256, y: __m256) -> __m256 {
        let mut t = log(x);
        t = _mm256_mul_ps(t, y);
        t = exp(t);

        // this doesn't work for x==0, so we have to set t to 0 where x==0
        let c = _mm256_cmp_ps(x, _mm256_setzero_ps(), _CMP_EQ_OQ);
        t = _mm256_andnot_ps(c, t);

        t
    }

    const fn const_f32(x: f32) -> __m256 {
        // https://www.reddit.com/r/rust/comments/8ltns0/how_to_set_some_simd_constants/
        #[repr(C)]
        union U {
            a: __m256,
            b: [f32; 8],
        }
        unsafe {
            U {
                b: [x, x, x, x, x, x, x, x],
            }
            .a
        }
    }
    const fn const_i32(x: i32) -> __m256i {
        #[repr(C)]
        union U {
            a: __m256i,
            b: [i32; 8],
        }
        unsafe {
            U {
                b: [x, x, x, x, x, x, x, x],
            }
            .a
        }
    }
    fn bit_cast_i32_to_f32(x: __m256i) -> __m256 {
        #[repr(C)]
        union U {
            a: __m256,
            b: __m256i,
        }
        unsafe { U { b: x }.a }
    }
    fn bit_cast_f32_to_i32(x: __m256) -> __m256i {
        #[repr(C)]
        union U {
            a: __m256,
            b: __m256i,
        }
        unsafe { U { a: x }.b }
    }

    // The following code is adapted from
    // https://github.com/yuyichao/avx2_mathfun/blob/3c48718fd7fa4f427906e59d43e7cc1ef69cc276/avx2_mathfun.h
    //
    // Copyright (C) 2012 Giovanni Garberoglio
    // Interdisciplinary Laboratory for Computational Science (LISC)
    // Fondazione Bruno Kessler and University of Trento
    // via Sommarive, 18
    // I-38123 Trento (Italy)
    // zlib license

    const ONE: __m256 = const_f32(1.0);
    const HALF: __m256 = const_f32(0.5);

    const X7F: __m256i = const_i32(0x7F);

    #[target_feature(enable = "avx2")]
    pub unsafe fn exp(mut x: __m256) -> __m256 {
        const EXP_HI: __m256 = const_f32(88.3762626647949);
        const EXP_LO: __m256 = const_f32(-88.3762626647949);

        const LOG2EF: __m256 = const_f32(1.44269504088896341);
        const EXP_C1: __m256 = const_f32(0.693359375);
        const EXP_C2: __m256 = const_f32(-2.12194440e-4);

        const EXP_P0: __m256 = const_f32(1.9875691500E-4);
        const EXP_P1: __m256 = const_f32(1.3981999507E-3);
        const EXP_P2: __m256 = const_f32(8.3334519073E-3);
        const EXP_P3: __m256 = const_f32(4.1665795894E-2);
        const EXP_P4: __m256 = const_f32(1.6666665459E-1);
        const EXP_P5: __m256 = const_f32(5.0000001201E-1);

        x = _mm256_min_ps(x, EXP_HI);
        x = _mm256_max_ps(x, EXP_LO);

        /* express exp(x) as exp(g + n*log(2)) */
        let mut fx = _mm256_mul_ps(x, LOG2EF);
        fx = _mm256_add_ps(fx, HALF);

        /* how to perform a floorf with SSE: just below */
        // imm0 = _mm256_cvttps_epi32(fx);
        // tmp  = _mm256_cvtepi32_ps(imm0);

        let tmp = _mm256_floor_ps(fx);

        /* if greater, substract 1 */
        let mut mask = _mm256_cmp_ps(tmp, fx, _CMP_GT_OS);
        mask = _mm256_and_ps(mask, ONE);
        fx = _mm256_sub_ps(tmp, mask);

        let mut z = _mm256_mul_ps(fx, EXP_C2);
        x = _mm256_sub_ps(x, _mm256_mul_ps(fx, EXP_C1));
        x = _mm256_sub_ps(x, z);

        z = _mm256_mul_ps(x, x);

        let mut y = _mm256_mul_ps(EXP_P0, x);
        y = _mm256_add_ps(y, EXP_P1);
        y = _mm256_mul_ps(y, x);
        y = _mm256_add_ps(y, EXP_P2);
        y = _mm256_mul_ps(y, x);
        y = _mm256_add_ps(y, EXP_P3);
        y = _mm256_mul_ps(y, x);
        y = _mm256_add_ps(y, EXP_P4);
        y = _mm256_mul_ps(y, x);
        y = _mm256_add_ps(y, EXP_P5);
        y = _mm256_mul_ps(y, z);
        y = _mm256_add_ps(y, x);
        y = _mm256_add_ps(y, ONE);

        /* build 2^n */
        let mut imm0 = _mm256_cvttps_epi32(fx);
        // another two AVX2 instructions
        imm0 = _mm256_add_epi32(imm0, X7F);
        imm0 = _mm256_slli_epi32(imm0, 23);
        y = _mm256_mul_ps(y, bit_cast_i32_to_f32(imm0));
        y
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn log(mut x: __m256) -> __m256 {
        const MIN_NORM_POS: __m256i = const_i32(0x00800000);
        const INV_MANT_MASK: __m256i = const_i32(!0x7f800000);

        const SQRTHF: __m256 = const_f32(0.707106781186547524);
        const LOG_P0: __m256 = const_f32(7.0376836292E-2);
        const LOG_P1: __m256 = const_f32(-1.1514610310E-1);
        const LOG_P2: __m256 = const_f32(1.1676998740E-1);
        const LOG_P3: __m256 = const_f32(-1.2420140846E-1);
        const LOG_P4: __m256 = const_f32(1.4249322787E-1);
        const LOG_P5: __m256 = const_f32(-1.6668057665E-1);
        const LOG_P6: __m256 = const_f32(2.0000714765E-1);
        const LOG_P7: __m256 = const_f32(-2.4999993993E-1);
        const LOG_P8: __m256 = const_f32(3.3333331174E-1);
        const LOG_Q1: __m256 = const_f32(-2.12194440e-4);
        const LOG_Q2: __m256 = const_f32(0.693359375);

        let invalid_mask = _mm256_cmp_ps(x, _mm256_setzero_ps(), _CMP_LE_OS);

        x = _mm256_max_ps(x, bit_cast_i32_to_f32(MIN_NORM_POS)); /* cut off denormalized stuff */

        // can be done with AVX2
        let mut imm0 = _mm256_srli_epi32(bit_cast_f32_to_i32(x), 23);

        /* keep only the fractional part */
        x = _mm256_and_ps(x, bit_cast_i32_to_f32(INV_MANT_MASK));
        x = _mm256_or_ps(x, HALF);

        // this is again another AVX2 instruction
        imm0 = _mm256_sub_epi32(imm0, X7F);
        let mut e = _mm256_cvtepi32_ps(imm0);

        e = _mm256_add_ps(e, ONE);

        /* part2:
           if( x < SQRTHF ) {
           e -= 1;
           x = x + x - 1.0;
           } else { x = x - 1.0; }
        */
        let mask = _mm256_cmp_ps(x, SQRTHF, _CMP_LT_OS);
        let tmp = _mm256_and_ps(x, mask);
        x = _mm256_sub_ps(x, ONE);
        e = _mm256_sub_ps(e, _mm256_and_ps(ONE, mask));
        x = _mm256_add_ps(x, tmp);

        let z = _mm256_mul_ps(x, x);

        let mut y = _mm256_mul_ps(LOG_P0, x);
        y = _mm256_add_ps(y, LOG_P1);
        y = _mm256_mul_ps(y, x);
        y = _mm256_add_ps(y, LOG_P2);
        y = _mm256_mul_ps(y, x);
        y = _mm256_add_ps(y, LOG_P3);
        y = _mm256_mul_ps(y, x);
        y = _mm256_add_ps(y, LOG_P4);
        y = _mm256_mul_ps(y, x);
        y = _mm256_add_ps(y, LOG_P5);
        y = _mm256_mul_ps(y, x);
        y = _mm256_add_ps(y, LOG_P6);
        y = _mm256_mul_ps(y, x);
        y = _mm256_add_ps(y, LOG_P7);
        y = _mm256_mul_ps(y, x);
        y = _mm256_add_ps(y, LOG_P8);
        y = _mm256_mul_ps(y, x);

        y = _mm256_mul_ps(y, z);

        y = _mm256_add_ps(y, _mm256_mul_ps(e, LOG_Q1));

        y = _mm256_sub_ps(y, _mm256_mul_ps(z, HALF));

        x = _mm256_add_ps(x, y);
        x = _mm256_add_ps(x, _mm256_mul_ps(e, LOG_Q2));
        x = _mm256_or_ps(x, invalid_mask); // negative arg will be NAN
        x
    }
}

#[cfg(test)]
mod tests {
    use image_core::NDimImage;
    use test_util::{
        data::{read_flower_transparent, read_portrait},
        snap::ImageSnapshot,
    };

    #[test]
    fn gamma() {
        let mut img: NDimImage = read_flower_transparent().into();
        super::gamma_ndim(&mut img, 2.2);
        img.snapshot("gamma_rgba");

        let mut img: NDimImage = read_portrait().into();
        super::gamma_ndim(&mut img, 2.2);
        img.snapshot("gamma_rgb");
    }
}
