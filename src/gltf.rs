use gltf::{Node, Scene};
use gltf::camera::Projection;
use gltf::scene::iter;

use crate::object;
use crate::object::Transform;

pub fn extract<T: Clone>(
    scene: &Scene,
    parse_fn: fn(&Node, Transform) -> Option<T>,
) -> Option<T> {
    for node in scene.nodes() {
        let carry = object::Transform::from(node.transform());

        let maybe_object = parse_fn(&node, carry);
        if maybe_object.is_some() {
            return maybe_object;
        }

        let objects = visit_nodes(
            node.children(),
            carry,
            parse_fn,
            true,
        );
        if !objects.is_empty() {
            return objects.get(0).cloned();
        }
    }

    None
}

pub fn extract_all<T>(
    scene: &Scene,
    parse_fn: fn(&Node, Transform) -> Option<T>,
) -> Vec<T> {
    let mut result = vec![];
    for node in scene.nodes() {
        let carry = object::Transform::from(node.transform());

        let maybe_object = parse_fn(&node, carry);
        if let Some(object) = maybe_object {
            result.push(object);
        }

        let objects = visit_nodes(
            node.children(),
            carry,
            parse_fn,
            false,
        );

        if !objects.is_empty() {
            result.extend(objects);
        }
    }

    result
}

pub fn get_camera(node: &Node, carry: Transform) -> Option<object::Camera> {
    if let Some(camera) = node.camera() {
        match camera.projection() {
            Projection::Perspective(perspective) => {
                return Some(object::Camera {
                    parent_transform: carry,
                    transform: object::Transform::from(node.transform()),
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

pub fn get_mesh(node: &Node, carry: Transform) -> Option<object::Mesh> {
    if let Some(_) = node.mesh() {
        return Some(object::Mesh {
            parent_transform: carry,
            transform: object::Transform::from(node.transform()),
        });
    }
    None
}

fn visit_nodes<T>(
    nodes: iter::Children,
    carry: Transform,
    parse_fn: fn(&Node, Transform) -> Option<T>,
    break_on_first: bool,
) -> Vec<T> {
    let mut result = vec![];
    for node in nodes {
        if let Some(mesh) = parse_fn(&node, carry) {
            result.push(mesh);
            if break_on_first {
                return result;
            }
        }

        let carry = carry * object::Transform::from(node.transform());

        let objects = visit_nodes(
            node.children(),
            carry,
            parse_fn,
            break_on_first,
        );
        if !objects.is_empty() {
            result.extend(objects);
            if break_on_first {
                return result;
            }
        }
    }

    result
}
