use nalgebra::{Point3, Scale3, UnitQuaternion};

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Scale3<f32>,
    pub aspect_ratio: f32,
    pub yfov: f32,
    pub zfar: f32,
    pub znear: f32,
}

#[derive(Debug)]
pub struct Mesh {
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Scale3<f32>,
}
