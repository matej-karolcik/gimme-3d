use gltf::{Node, Scene};
use gltf::camera::Projection;
use gltf::scene::iter;

use crate::object;
use crate::object::Transform;

pub fn extract_camera(scene: &Scene) -> Option<object::Camera> {
    for node in scene.nodes() {
        let carry = object::Transform::from(node.transform());
        let maybe_camera = get_camera(&node, carry);
        if maybe_camera.is_some() {
            return maybe_camera;
        }
        let maybe_camera = visit_nodes(node.children(), carry);
        if maybe_camera.is_some() {
            return maybe_camera;
        }
    }

    None
}

fn get_camera(node: &Node, carry: Transform) -> Option<object::Camera> {
    if let Some(camera) = node.camera() {
        match camera.projection() {
            Projection::Perspective(perspective) => {
                return Some(object::Camera {
                    transform: carry * object::Transform::from(node.transform()),
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

fn visit_nodes(nodes: iter::Children, carry: Transform) -> Option<object::Camera> {
    for node in nodes {
        let carry = carry * object::Transform::from(node.transform());
        if let Some(camera) = get_camera(&node, carry) {
            return Some(camera);
        }
        let maybe_camera = visit_nodes(node.children(), carry);
        if maybe_camera.is_some() {
            return maybe_camera;
        }
    }

    None
}
