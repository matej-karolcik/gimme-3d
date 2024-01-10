use std::ops::Mul;

use cgmath::InnerSpace;
use cgmath::SquareMatrix;
use nalgebra::Matrix4;
use three_d::Vector4;
use three_d_asset::Mat4;

#[derive(Debug, Clone)]
pub struct Camera {
    pub parent_transform: Transform,
    pub transform: Transform,
    pub aspect_ratio: f32,
    pub yfov: f32,
    pub zfar: f32,
    pub znear: f32,
}

#[derive(Debug)]
pub struct Mesh {
    pub parent_transform: Transform,
    pub transform: Transform,
}

#[derive(Debug, Clone)]
pub struct Light {
    pub kind: LightKind,
    pub parent_transform: Transform,
    pub transform: Transform,
    // [0, 255]
    pub color: [u8; 3],
    pub intensity: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub matrix: Matrix4<f32>,
}

#[derive(Debug, Clone)]
pub enum LightKind {
    Directional,
    Point,
    Spot {
        inner_cone_angle: f32,
        outer_cone_angle: f32,
    },
}

impl From<gltf::khr_lights_punctual::Kind> for LightKind {
    fn from(kind: gltf::khr_lights_punctual::Kind) -> Self {
        match kind {
            gltf::khr_lights_punctual::Kind::Directional => Self::Directional,
            gltf::khr_lights_punctual::Kind::Point => Self::Point,
            gltf::khr_lights_punctual::Kind::Spot {
                inner_cone_angle,
                outer_cone_angle,
            } => Self::Spot {
                inner_cone_angle,
                outer_cone_angle,
            },
        }
    }
}

fn float_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.0001
}

impl Transform {
    pub fn has_equal_rotation(&self, other: &Self) -> bool {
        let (_, r1, _) = self.decomposed();
        let (_, r2, _) = other.decomposed();
        float_eq(r1[0], r2[0])
            && float_eq(r1[1], r2[1])
            && float_eq(r1[2], r2[2])
            && float_eq(r1[3], r2[3])
    }

    pub fn from_quaternion(quaternion: nalgebra::Quaternion<f32>) -> Self {
        let t = gltf::scene::Transform::Decomposed {
            translation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            rotation: [
                quaternion.coords.x,
                quaternion.coords.y,
                quaternion.coords.z,
                quaternion.coords.w,
            ],
        };
        Self::from(t)
    }

    pub fn decomposed(&self) -> ([f32; 3], [f32; 4], [f32; 3]) {
        let translation = [self.matrix.m41, self.matrix.m42, self.matrix.m43];
        #[rustfmt::skip]
            let mut i = cgmath::Matrix3::new(
            self.matrix.m11, self.matrix.m21, self.matrix.m31,
            self.matrix.m12, self.matrix.m22, self.matrix.m32,
            self.matrix.m13, self.matrix.m23, self.matrix.m33,
        );
        let sx = i.x.magnitude();
        let sy = i.y.magnitude();
        let sz = i.determinant().signum() * i.z.magnitude();

        let scale = [sx, sy, sz];

        i.x = i.x.mul(1.0 / sx);
        i.y = i.y.mul(1.0 / sy);
        i.z = i.z.mul(1.0 / sz);

        let r = cgmath::Quaternion::from(i);
        let rotation = [r.v.x, r.v.y, r.v.z, r.s];

        (translation, rotation, scale)
    }
}

impl From<gltf::scene::Transform> for Transform {
    fn from(transform: gltf::scene::Transform) -> Self {
        Self {
            matrix: transform.matrix().into(),
        }
    }
}

impl From<Mat4> for Transform {
    fn from(value: Mat4) -> Self {
        Self {
            matrix: Matrix4::new(
                value.x.x, value.y.x, value.z.x, value.w.x,
                value.x.y, value.y.y, value.z.y, value.w.y,
                value.x.z, value.y.z, value.z.z, value.w.z,
                value.x.w, value.y.w, value.z.w, value.w.w,
            ),
        }
    }
}

impl Into<Mat4> for Transform {
    fn into(self) -> Mat4 {
        let x = self.matrix.column(0);
        let y = self.matrix.column(1);
        let z = self.matrix.column(2);
        let w = self.matrix.column(3);

        Mat4::from_cols(
            Vector4::new(x.x, x.y, x.z, x.w),
            Vector4::new(y.x, y.y, y.z, y.w),
            Vector4::new(z.x, z.y, z.z, z.w),
            Vector4::new(w.x, w.y, w.z, w.w),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_eq() {
        assert!(float_eq(0.0, 0.0));
        assert!(float_eq(0.0001, 0.0001));
        assert!(float_eq(-3.0, -3.0));
        assert_eq!(float_eq(0.0001, 0.0002), false);
    }
}
