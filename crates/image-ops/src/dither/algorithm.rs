pub trait Diffuser {
    fn assign_weight(&mut self, y: usize, x: isize, weight: f32);
}
pub trait DiffusionAlgorithm {
    fn define_weights(&self, diffuser: impl Diffuser);
}

pub struct Atkinson;
impl DiffusionAlgorithm for Atkinson {
    fn define_weights(&self, mut diffuser: impl Diffuser) {
        diffuser.assign_weight(0, 1, 1_f32 / 8_f32);
        diffuser.assign_weight(0, 2, 1_f32 / 8_f32);

        diffuser.assign_weight(1, -1, 1_f32 / 8_f32);
        diffuser.assign_weight(1, 0, 1_f32 / 8_f32);
        diffuser.assign_weight(1, 1, 1_f32 / 8_f32);

        diffuser.assign_weight(2, 0, 1_f32 / 8_f32);
    }
}

pub struct Burkes;
impl DiffusionAlgorithm for Burkes {
    fn define_weights(&self, mut diffuser: impl Diffuser) {
        diffuser.assign_weight(0, 1, 8_f32 / 32_f32);
        diffuser.assign_weight(0, 2, 4_f32 / 32_f32);

        diffuser.assign_weight(1, -2, 2_f32 / 32_f32);
        diffuser.assign_weight(1, -1, 4_f32 / 32_f32);
        diffuser.assign_weight(1, 0, 8_f32 / 32_f32);
        diffuser.assign_weight(1, 1, 4_f32 / 32_f32);
        diffuser.assign_weight(1, 2, 2_f32 / 32_f32);
    }
}

pub struct FloydSteinberg;
impl DiffusionAlgorithm for FloydSteinberg {
    fn define_weights(&self, mut diffuser: impl Diffuser) {
        diffuser.assign_weight(0, 1, 7_f32 / 16_f32);

        diffuser.assign_weight(1, -1, 3_f32 / 16_f32);
        diffuser.assign_weight(1, 0, 5_f32 / 16_f32);
        diffuser.assign_weight(1, 1, 1_f32 / 16_f32);
    }
}

pub struct JarvisJudiceNinke;
impl DiffusionAlgorithm for JarvisJudiceNinke {
    fn define_weights(&self, mut diffuser: impl Diffuser) {
        diffuser.assign_weight(0, 1, 7_f32 / 48_f32);
        diffuser.assign_weight(0, 2, 5_f32 / 48_f32);

        diffuser.assign_weight(1, -2, 3_f32 / 48_f32);
        diffuser.assign_weight(1, -1, 5_f32 / 48_f32);
        diffuser.assign_weight(1, 0, 7_f32 / 48_f32);
        diffuser.assign_weight(1, 1, 5_f32 / 48_f32);
        diffuser.assign_weight(1, 2, 3_f32 / 48_f32);

        diffuser.assign_weight(2, -2, 1_f32 / 48_f32);
        diffuser.assign_weight(2, -1, 3_f32 / 48_f32);
        diffuser.assign_weight(2, 0, 5_f32 / 48_f32);
        diffuser.assign_weight(2, 1, 3_f32 / 48_f32);
        diffuser.assign_weight(2, 2, 1_f32 / 48_f32);
    }
}

pub struct Sierra;
impl DiffusionAlgorithm for Sierra {
    fn define_weights(&self, mut diffuser: impl Diffuser) {
        diffuser.assign_weight(0, 1, 5_f32 / 32_f32);
        diffuser.assign_weight(0, 2, 3_f32 / 32_f32);

        diffuser.assign_weight(1, -2, 2_f32 / 32_f32);
        diffuser.assign_weight(1, -1, 4_f32 / 32_f32);
        diffuser.assign_weight(1, 0, 5_f32 / 32_f32);
        diffuser.assign_weight(1, 1, 4_f32 / 32_f32);
        diffuser.assign_weight(1, 2, 2_f32 / 32_f32);

        diffuser.assign_weight(2, -1, 2_f32 / 32_f32);
        diffuser.assign_weight(2, 0, 3_f32 / 32_f32);
        diffuser.assign_weight(2, 1, 2_f32 / 32_f32);
    }
}

pub struct TwoRowSierra;
impl DiffusionAlgorithm for TwoRowSierra {
    fn define_weights(&self, mut diffuser: impl Diffuser) {
        diffuser.assign_weight(0, 1, 4_f32 / 16_f32);
        diffuser.assign_weight(0, 2, 3_f32 / 16_f32);

        diffuser.assign_weight(1, -2, 1_f32 / 16_f32);
        diffuser.assign_weight(1, -1, 2_f32 / 16_f32);
        diffuser.assign_weight(1, 0, 3_f32 / 16_f32);
        diffuser.assign_weight(1, 1, 2_f32 / 16_f32);
        diffuser.assign_weight(1, 2, 1_f32 / 16_f32);
    }
}

pub struct SierraLite;
impl DiffusionAlgorithm for SierraLite {
    fn define_weights(&self, mut diffuser: impl Diffuser) {
        diffuser.assign_weight(0, 1, 2_f32 / 4_f32);

        diffuser.assign_weight(1, -1, 1_f32 / 4_f32);
        diffuser.assign_weight(1, 0, 1_f32 / 4_f32);
    }
}

pub struct Stucki;
impl DiffusionAlgorithm for Stucki {
    fn define_weights(&self, mut diffuser: impl Diffuser) {
        diffuser.assign_weight(0, 1, 8_f32 / 42_f32);
        diffuser.assign_weight(0, 2, 4_f32 / 42_f32);

        diffuser.assign_weight(1, -2, 2_f32 / 42_f32);
        diffuser.assign_weight(1, -1, 4_f32 / 42_f32);
        diffuser.assign_weight(1, 0, 8_f32 / 42_f32);
        diffuser.assign_weight(1, 1, 4_f32 / 42_f32);
        diffuser.assign_weight(1, 2, 2_f32 / 42_f32);

        diffuser.assign_weight(2, -2, 1_f32 / 42_f32);
        diffuser.assign_weight(2, -1, 2_f32 / 42_f32);
        diffuser.assign_weight(2, 0, 4_f32 / 42_f32);
        diffuser.assign_weight(2, 1, 2_f32 / 42_f32);
        diffuser.assign_weight(2, 2, 1_f32 / 42_f32);
    }
}
