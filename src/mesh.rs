use gltf::{Node, Scene};
use gltf::scene::iter;
use nalgebra::{Point3, Quaternion, Scale3, UnitQuaternion};

use crate::object;

pub fn extract_mesh(scene: &Scene) -> Option<object::Mesh> {
    for node in scene.nodes() {
        let maybe_mesh = get_mesh(&node);
        if maybe_mesh.is_some() {
            return maybe_mesh;
        }
        let maybe_mesh = visit_nodes(node.children());
        if maybe_mesh.is_some() {
            return maybe_mesh;
        }
    }

    None
}

fn get_mesh(node: &Node) -> Option<object::Mesh> {
    if let Some(_) = node.mesh() {
        let (position, rotation, scale) = node.transform().decomposed();
        return Some(object::Mesh {
            position: Point3::new(
                position[0],
                position[1],
                position[2],
            ),
            rotation: UnitQuaternion::from_quaternion(Quaternion::new(
                rotation[3],
                rotation[0],
                rotation[1],
                rotation[2],
            )),
            scale: Scale3::new(
                scale[0],
                scale[1],
                scale[2],
            ),
        });
    }
    None
}

fn visit_nodes(nodes: iter::Children) -> Option<object::Mesh> {
    for node in nodes {
        if let Some(mesh) = get_mesh(&node) {
            return Some(mesh);
        }
        let maybe_mesh = visit_nodes(node.children());
        if maybe_mesh.is_some() {
            return maybe_mesh;
        }
    }

    None
}
