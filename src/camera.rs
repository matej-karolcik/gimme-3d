use gltf::{Node, Scene};
use gltf::camera::Projection;
use gltf::scene::iter;
use nalgebra::{Quaternion, Vector3};

use crate::object;

pub fn extract_camera(scene: &Scene) -> Option<object::Camera> {
    for node in scene.nodes() {
        let maybe_camera = get_camera(&node);
        if maybe_camera.is_some() {
            return maybe_camera;
        }
        let maybe_camera = visit_nodes(node.children());
        if maybe_camera.is_some() {
            return maybe_camera;
        }
    }

    None
}

fn get_camera(node: &Node) -> Option<object::Camera> {
    if let Some(camera) = node.camera() {
        let (position, rotation, scale) = node.transform().decomposed();
        match camera.projection() {
            Projection::Perspective(perspective) => {
                return Some(object::Camera {
                    position: Vector3::new(position[0], position[1], position[2]),
                    rotation: Quaternion::new(rotation[3], rotation[0], rotation[1], rotation[2]),
                    scale: Vector3::new(scale[0], scale[1], scale[2]),
                    aspect_ratio: perspective.aspect_ratio().unwrap_or(1.0),
                    yfov: perspective.yfov(),
                    zfar: perspective.zfar().unwrap_or(100.0),
                    znear: perspective.znear(),
                });
            }
            _ => {}
        }
    }
    None
}

fn visit_nodes(nodes: iter::Children) -> Option<object::Camera> {
    for node in nodes {
        if let Some(camera) = get_camera(&node) {
            return Some(camera);
        }
        let maybe_camera = visit_nodes(node.children());
        if maybe_camera.is_some() {
            return maybe_camera;
        }
    }

    None
}
