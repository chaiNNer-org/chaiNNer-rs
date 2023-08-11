pub trait Pixel: Copy + Default + std::ops::AddAssign + std::ops::Mul<f32, Output = Self> {}
impl<P> Pixel for P where P: Copy + Default + std::ops::AddAssign + std::ops::Mul<f32, Output = Self>
{}
