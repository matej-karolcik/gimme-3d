use nalgebra::{Quaternion, Vector3};

#[derive(Debug)]
pub struct Camera {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    pub aspect_ratio: f32,
    pub yfov: f32,
    pub zfar: f32,
    pub znear: f32,
}

#[derive(Debug)]
pub struct Mesh {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}
