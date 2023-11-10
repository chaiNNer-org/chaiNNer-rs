use std::ops::{Add, Mul};

// License: GNU Lesser GPL
// Code translated from https://code.google.com/archive/p/hqx/

#[inline]
pub fn interp1<T: Add<T, Output = T> + Mul<f32, Output = T>>(a: T, b: T) -> T {
    (a * 3.0 + b) * 0.25
}
#[inline]
pub fn interp2<T: Add<T, Output = T> + Mul<f32, Output = T>>(a: T, b: T, c: T) -> T {
    (a * 2.0 + b + c) * 0.25
}
#[inline]
pub fn interp3<T: Add<T, Output = T> + Mul<f32, Output = T>>(a: T, b: T) -> T {
    (a * 7.0 + b) * 0.125
}
#[inline]
pub fn interp4<T: Add<T, Output = T> + Mul<f32, Output = T>>(a: T, b: T, c: T) -> T {
    (a * 2.0 + (b + c) * 7.0) * 0.0625
}
#[inline]
pub fn interp5<T: Add<T, Output = T> + Mul<f32, Output = T>>(a: T, b: T) -> T {
    (a + b) * 0.5
}
#[inline]
pub fn interp6<T: Add<T, Output = T> + Mul<f32, Output = T>>(a: T, b: T, c: T) -> T {
    (a * 5.0 + b * 2.0 + c) * 0.125
}
#[inline]
pub fn interp7<T: Add<T, Output = T> + Mul<f32, Output = T>>(a: T, b: T, c: T) -> T {
    (a * 6.0 + b + c) * 0.125
}
#[inline]
pub fn interp8<T: Add<T, Output = T> + Mul<f32, Output = T>>(a: T, b: T) -> T {
    (a * 5.0 + b * 3.0) * 0.125
}
#[inline]
pub fn interp9<T: Add<T, Output = T> + Mul<f32, Output = T>>(a: T, b: T, c: T) -> T {
    (a * 2.0 + (b + c) * 3.0) * 0.125
}
#[inline]
pub fn interp10<T: Add<T, Output = T> + Mul<f32, Output = T>>(a: T, b: T, c: T) -> T {
    (a * 14.0 + b + c) * 0.0625
}
