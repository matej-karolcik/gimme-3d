use gltf::{Node, Scene};
use gltf::scene::iter;

use crate::object;
use crate::object::Transform;

pub fn extract_mesh(scene: &Scene) -> Option<object::Mesh> {
    for node in scene.nodes() {
        let transform = object::Transform::from(node.transform());
        let maybe_mesh = get_mesh(&node, transform);
        if maybe_mesh.is_some() {
            return maybe_mesh;
        }
        let maybe_mesh = visit_nodes(node.children(), transform);
        if maybe_mesh.is_some() {
            return maybe_mesh;
        }
    }

    None
}

fn get_mesh(node: &Node, carry: Transform) -> Option<object::Mesh> {
    if let Some(_) = node.mesh() {
        return Some(object::Mesh {
            transform: carry * object::Transform::from(node.transform()),
        });
    }
    None
}

fn visit_nodes(nodes: iter::Children, carry: Transform) -> Option<object::Mesh> {
    for node in nodes {
        let carry = carry * object::Transform::from(node.transform());
        if let Some(mesh) = get_mesh(&node, carry) {
            return Some(mesh);
        }
        let maybe_mesh = visit_nodes(node.children(), carry);
        if maybe_mesh.is_some() {
            return maybe_mesh;
        }
    }

    None
}
