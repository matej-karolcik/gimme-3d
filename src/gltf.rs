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

        let maybe_camera = visit_nodes_t(
            node.children(),
            carry,
            get_camera,
        );
        if maybe_camera.is_some() {
            return maybe_camera;
        }
    }

    None
}

pub fn extract<T>(
    scene: &Scene,
    parse_fn: fn(&Node, Transform) -> Option<T>,
) -> Option<T> {
    for node in scene.nodes() {
        let carry = object::Transform::from(node.transform());

        let maybe_object = parse_fn(&node, carry);
        if maybe_object.is_some() {
            return maybe_object;
        }

        let maybe_object = visit_nodes_t(
            node.children(),
            carry,
            parse_fn,
        );
        if maybe_object.is_some() {
            return maybe_object;
        }
    }

    None
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

pub fn extract_mesh(scene: &Scene) -> Option<object::Mesh> {
    for node in scene.nodes() {
        let transform = object::Transform::from(node.transform());

        let maybe_mesh = get_mesh(&node, transform);
        if maybe_mesh.is_some() {
            return maybe_mesh;
        }

        let maybe_mesh = visit_nodes_t(
            node.children(),
            transform,
            get_mesh,
        );
        if maybe_mesh.is_some() {
            return maybe_mesh;
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

fn visit_nodes_t<T>(
    nodes: iter::Children,
    carry: Transform,
    parse_fn: fn(&Node, Transform) -> Option<T>,
) -> Option<T> {
    for node in nodes {
        if let Some(mesh) = parse_fn(&node, carry) {
            return Some(mesh);
        }

        let carry = carry * object::Transform::from(node.transform());

        let maybe_mesh = visit_nodes_t(
            node.children(),
            carry,
            parse_fn,
        );
        if maybe_mesh.is_some() {
            return maybe_mesh;
        }
    }

    None
}
