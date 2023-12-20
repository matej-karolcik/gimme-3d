use std::ops::Mul;

use nalgebra::Matrix4;
use three_d_asset::Mat4;

#[derive(Debug)]
pub struct Camera {
    pub transform: Transform,
    pub aspect_ratio: f32,
    pub yfov: f32,
    pub zfar: f32,
    pub znear: f32,
}

#[derive(Debug)]
pub struct Mesh {
    pub transform: Transform,
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub matrix: Matrix4<f32>,
}

impl From<gltf::scene::Transform> for Transform {
    fn from(transform: gltf::scene::Transform) -> Self {
        Self {
            matrix: transform.matrix().into(),
        }
    }
}

impl Into<Mat4> for Transform {
    fn into(self) -> Mat4 {
        Mat4::new(
            self.matrix.m11, self.matrix.m21, self.matrix.m31, self.matrix.m41,
            self.matrix.m12, self.matrix.m22, self.matrix.m32, self.matrix.m42,
            self.matrix.m13, self.matrix.m23, self.matrix.m33, self.matrix.m43,
            self.matrix.m14, self.matrix.m24, self.matrix.m34, self.matrix.m44,
        )
    }
}

impl Mul for Transform {
    type Output = Transform;

    fn mul(self, rhs: Transform) -> Self::Output {
        Self {
            matrix: self.matrix * rhs.matrix,
        }
    }
}
