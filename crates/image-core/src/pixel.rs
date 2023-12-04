use glam::{Vec2, Vec3, Vec3A, Vec4};

use crate::util::vec_into_flattened;

pub trait Components {
    const COMPONENTS: usize;
}
impl Components for f32 {
    const COMPONENTS: usize = 1;
}
impl<const N: usize> Components for [f32; N] {
    const COMPONENTS: usize = N;
}
impl Components for (f32, f32) {
    const COMPONENTS: usize = 2;
}
impl Components for (f32, f32, f32) {
    const COMPONENTS: usize = 3;
}
impl Components for (f32, f32, f32, f32) {
    const COMPONENTS: usize = 4;
}
impl Components for Vec2 {
    const COMPONENTS: usize = 2;
}
impl Components for Vec3 {
    const COMPONENTS: usize = 3;
}
impl Components for Vec3A {
    const COMPONENTS: usize = 3;
}
impl Components for Vec4 {
    const COMPONENTS: usize = 4;
}

pub trait FlattenData: Components + Sized {
    fn flatten_data(vec: Vec<Self>) -> Vec<f32>;
}

impl FlattenData for f32 {
    fn flatten_data(vec: Vec<Self>) -> Vec<f32> {
        vec
    }
}
impl<const N: usize> FlattenData for [f32; N] {
    fn flatten_data(vec: Vec<Self>) -> Vec<f32> {
        vec_into_flattened(vec)
    }
}
impl FlattenData for (f32, f32) {
    fn flatten_data(vec: Vec<Self>) -> Vec<f32> {
        let vec: Vec<_> = vec.into_iter().map(|x| x.into()).collect();
        vec_into_flattened(vec)
    }
}
impl FlattenData for (f32, f32, f32) {
    fn flatten_data(vec: Vec<Self>) -> Vec<f32> {
        let vec: Vec<_> = vec.into_iter().map(|x| x.into()).collect();
        vec_into_flattened(vec)
    }
}
impl FlattenData for (f32, f32, f32, f32) {
    fn flatten_data(vec: Vec<Self>) -> Vec<f32> {
        let vec: Vec<_> = vec.into_iter().map(|x| x.into()).collect();
        vec_into_flattened(vec)
    }
}
impl FlattenData for Vec2 {
    fn flatten_data(vec: Vec<Self>) -> Vec<f32> {
        let vec: Vec<_> = vec.into_iter().map(|x| x.into()).collect();
        vec_into_flattened(vec)
    }
}
impl FlattenData for Vec3 {
    fn flatten_data(vec: Vec<Self>) -> Vec<f32> {
        let vec: Vec<_> = vec.into_iter().map(|x| x.into()).collect();
        vec_into_flattened(vec)
    }
}
impl FlattenData for Vec3A {
    fn flatten_data(vec: Vec<Self>) -> Vec<f32> {
        let vec: Vec<_> = vec.into_iter().map(|x| x.into()).collect();
        vec_into_flattened(vec)
    }
}
impl FlattenData for Vec4 {
    fn flatten_data(vec: Vec<Self>) -> Vec<f32> {
        let vec: Vec<_> = vec.into_iter().map(|x| x.into()).collect();
        vec_into_flattened(vec)
    }
}

pub trait ClipFloat {
    fn clip(self, min: f32, max: f32) -> Self;
}

impl ClipFloat for f32 {
    fn clip(self, min: f32, max: f32) -> Self {
        self.clamp(min, max)
    }
}
impl<const N: usize> ClipFloat for [f32; N] {
    fn clip(self, min: f32, max: f32) -> Self {
        self.map(|x| x.clamp(min, max))
    }
}
impl ClipFloat for (f32, f32) {
    fn clip(self, min: f32, max: f32) -> Self {
        (self.0.clamp(min, max), self.1.clamp(min, max))
    }
}
impl ClipFloat for (f32, f32, f32) {
    fn clip(self, min: f32, max: f32) -> Self {
        (
            self.0.clamp(min, max),
            self.1.clamp(min, max),
            self.2.clamp(min, max),
        )
    }
}
impl ClipFloat for (f32, f32, f32, f32) {
    fn clip(self, min: f32, max: f32) -> Self {
        (
            self.0.clamp(min, max),
            self.1.clamp(min, max),
            self.2.clamp(min, max),
            self.3.clamp(min, max),
        )
    }
}
impl ClipFloat for Vec2 {
    fn clip(self, min: f32, max: f32) -> Self {
        self.clamp(Self::splat(min), Self::splat(max))
    }
}
impl ClipFloat for Vec3 {
    fn clip(self, min: f32, max: f32) -> Self {
        self.clamp(Self::splat(min), Self::splat(max))
    }
}
impl ClipFloat for Vec3A {
    fn clip(self, min: f32, max: f32) -> Self {
        self.clamp(Self::splat(min), Self::splat(max))
    }
}
impl ClipFloat for Vec4 {
    fn clip(self, min: f32, max: f32) -> Self {
        self.clamp(Self::splat(min), Self::splat(max))
    }
}
