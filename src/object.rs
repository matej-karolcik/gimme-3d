use std::ops::Mul;

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

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub matrix: Matrix4<f32>,
}

fn float_compare(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.0001
}

impl Transform {
    pub fn has_equal_rotation(&self, other: &Self) -> bool {
        let (_, r1, _) = self.decomposed();
        let (_, r2, _) = other.decomposed();
        float_compare(r1[0], r2[0])
            && float_compare(r1[1], r2[1])
            && float_compare(r1[2], r2[2])
            && float_compare(r1[3], r2[3])
    }

    pub fn from_quaternion(quaternion: nalgebra::Quaternion<f32>) -> Self {
        let t = gltf::scene::Transform::Decomposed {
            translation: [0.0, 0.0, 0.0],
            rotation: [quaternion.coords.x, quaternion.coords.y, quaternion.coords.z, quaternion.coords.w],
            scale: [1.0, 1.0, 1.0],
        };
        Self::from(t)
    }

    pub fn decomposed(&self) -> ([f32; 3], [f32; 4], [f32; 3]) {
        let translation = [self.matrix.m41, self.matrix.m42, self.matrix.m43];
        #[rustfmt::skip]
            let mut i = Matrix3::new(
            self.matrix.m11, self.matrix.m21, self.matrix.m31,
            self.matrix.m12, self.matrix.m22, self.matrix.m32,
            self.matrix.m13, self.matrix.m23, self.matrix.m33,
        );
        let sx = i.x.magnitude();
        let sy = i.y.magnitude();
        let sz = i.determinant().signum() * i.z.magnitude();
        let scale = [sx, sy, sz];
        i.x.multiply(1.0 / sx);
        i.y.multiply(1.0 / sy);
        i.z.multiply(1.0 / sz);
        let r = Quaternion::from_matrix(i);
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

impl From<three_d::Mat4> for Transform {
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

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct Matrix3 {
    pub x: Vector3,
    pub y: Vector3,
    pub z: Vector3,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct Quaternion {
    pub s: f32,
    pub v: Vector3,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vector3 { x, y, z }
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn multiply(&mut self, s: f32) {
        self.x *= s;
        self.y *= s;
        self.z *= s;
    }
}

impl Quaternion {
    pub fn new(w: f32, xi: f32, yj: f32, zk: f32) -> Quaternion {
        Quaternion {
            s: w,
            v: Vector3::new(xi, yj, zk),
        }
    }

    #[cfg(test)]
    pub fn from_axis_angle(axis: Vector3, radians: f32) -> Quaternion {
        Quaternion {
            s: (0.5 * radians).cos(),
            v: axis * (0.5 * radians).sin(),
        }
    }

    /// Convert a rotation matrix to an equivalent quaternion.
    pub fn from_matrix(m: Matrix3) -> Quaternion {
        let trace = m.trace();
        if trace >= 0.0 {
            let s = (1.0 + trace).sqrt();
            let w = 0.5 * s;
            let s = 0.5 / s;
            let x = (m.y.z - m.z.y) * s;
            let y = (m.z.x - m.x.z) * s;
            let z = (m.x.y - m.y.x) * s;
            Quaternion::new(w, x, y, z)
        } else if (m.x.x > m.y.y) && (m.x.x > m.z.z) {
            let s = ((m.x.x - m.y.y - m.z.z) + 1.0).sqrt();
            let x = 0.5 * s;
            let s = 0.5 / s;
            let y = (m.y.x + m.x.y) * s;
            let z = (m.x.z + m.z.x) * s;
            let w = (m.y.z - m.z.y) * s;
            Quaternion::new(w, x, y, z)
        } else if m.y.y > m.z.z {
            let s = ((m.y.y - m.x.x - m.z.z) + 1.0).sqrt();
            let y = 0.5 * s;
            let s = 0.5 / s;
            let z = (m.z.y + m.y.z) * s;
            let x = (m.y.x + m.x.y) * s;
            let w = (m.z.x - m.x.z) * s;
            Quaternion::new(w, x, y, z)
        } else {
            let s = ((m.z.z - m.x.x - m.y.y) + 1.0).sqrt();
            let z = 0.5 * s;
            let s = 0.5 / s;
            let x = (m.x.z + m.z.x) * s;
            let y = (m.z.y + m.y.z) * s;
            let w = (m.x.y - m.y.x) * s;
            Quaternion::new(w, x, y, z)
        }
    }
}

impl Matrix3 {
    #[rustfmt::skip]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        c0r0: f32, c0r1: f32, c0r2: f32,
        c1r0: f32, c1r1: f32, c1r2: f32,
        c2r0: f32, c2r1: f32, c2r2: f32,
    ) -> Matrix3 {
        Matrix3 {
            x: Vector3::new(c0r0, c0r1, c0r2),
            y: Vector3::new(c1r0, c1r1, c1r2),
            z: Vector3::new(c2r0, c2r1, c2r2),
        }
    }

    pub fn determinant(&self) -> f32 {
        self.x.x * (self.y.y * self.z.z - self.z.y * self.y.z)
            - self.y.x * (self.x.y * self.z.z - self.z.y * self.x.z)
            + self.z.x * (self.x.y * self.y.z - self.y.y * self.x.z)
    }

    pub fn trace(&self) -> f32 {
        self.x.x + self.y.y + self.z.z
    }
}
