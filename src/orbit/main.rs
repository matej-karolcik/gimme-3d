use std::f32::consts::FRAC_1_SQRT_2;

use three_d::*;

use gimme_3d::error::Error;

#[tokio::main]
async fn main() {
    run().await;
}


pub async fn run() {
    let window = Window::new(WindowSettings {
        title: "Picking!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
        .unwrap();
    let context = window.gl();

    let model_slice = Vec::from(std::fs::read("assets/foobar.glb").unwrap());
    let gltf = gltf::Gltf::from_slice(model_slice.as_slice()).unwrap();
    let doc = gltf.document;

    let scene = doc.default_scene().ok_or(Error::NoDefaultScene).unwrap();
    let camera_props = gimme_3d::gltf::extract(&scene, gimme_3d::gltf::get_camera).ok_or(Error::NoCamera).unwrap();
    let mesh_props = gimme_3d::gltf::extract_all(&scene, gimme_3d::gltf::get_mesh);
    let light_props = gimme_3d::gltf::extract(&scene, gimme_3d::gltf::get_light).unwrap();

    let light_color = Srgba::from(light_props.color);
    let origin = nalgebra::Point3::origin();
    let light_transform = light_props.parent_transform * light_props.transform;
    let light_position = light_transform.matrix.transform_point(&origin);


    let camera_transform = camera_props.parent_transform * camera_props.transform;
    let origin = nalgebra::Point3::origin();
    let point = camera_transform.matrix.transform_point(&origin);
    let at = camera_props.parent_transform.matrix.transform_point(&origin);

    let light = Box::new(DirectionalLight::new(
        &context,
        light_props.intensity / 20.0,
        light_color,
        &vec3(point.x, point.y, point.z),
        // &vec3(light_position.x, light_position.y, light_position.z),
    ));

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(point.x, point.y, point.z),
        vec3(at.x, at.y, at.z),
        vec3(0.0, 1.0, 0.0),
        radians(camera_props.yfov * camera_props.aspect_ratio),
        0.01,
        camera_props.zfar / 100.,
        // vec3(4.0, 4.0, 5.0),
        // vec3(0.0, 0.0, 0.0),
        // vec3(0.0, 1.0, 0.0),
        // degrees(45.0),
        // 0.1,
        // 1000.0,
    );

    if (camera_props.parent_transform.decomposed().1[0].abs() - FRAC_1_SQRT_2).abs() < 0.0001
        && camera_props.transform.has_equal_rotation(&gimme_3d::object::Transform::from_quaternion(nalgebra::Quaternion::identity())) {
        camera.roll(three_d_asset::Deg::<f32>(90.0));
    }

    let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

    let mut sphere = CpuMesh::sphere(8);
    sphere.transform(&Mat4::from_scale(0.05)).unwrap();
    let mut pick_mesh = Gm::new(
        Mesh::new(&context, &sphere),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Srgba::RED,
                ..Default::default()
            },
        ),
    );

    let ambient = AmbientLight::new(&context, 0.4, Srgba::WHITE);
    let directional = DirectionalLight::new(&context, 2.0, Srgba::WHITE, &vec3(-1.0, -1.0, -1.0));

    let texture_bytes = std::fs::read("testdata/canvas.webp").unwrap();
    let mut texture = gimme_3d::img::decode_img(texture_bytes.as_slice()).unwrap();
    texture.data.to_linear_srgb();

    let mut loaded = three_d_asset::io::load_async(&["assets/foobar.glb"])
        .await
        .unwrap();

    let model = loaded.deserialize("assets/foobar.glb").unwrap();
    let mut monkey = Model::<PhysicalMaterial>::new(&context, &model).unwrap();
    monkey
        .iter_mut()
        .for_each(|m| {
            let mesh_props = mesh_props.get(0).unwrap();
            let transform = mesh_props.parent_transform * mesh_props.transform;
            m.set_transformation(transform.into());

            m.material.render_states.cull = Cull::None;
            m.material.albedo = Srgba::WHITE;
            m.material.is_transparent = true;
            m.material.render_states.blend = Blend::STANDARD_TRANSPARENCY;
            m.material.lighting_model = LightingModel::Phong;
            m.material.albedo_texture = Some(Texture2DRef::from_cpu_texture(&context, &texture));
        });


    // main loop
    window.render_loop(move |mut frame_input| {
        let mut change = frame_input.first_frame;
        change |= camera.set_viewport(frame_input.viewport);

        for event in frame_input.events.iter() {
            if let Event::MousePress {
                button, position, ..
            } = event
            {
                if *button == MouseButton::Left {
                    if let Some(pick) = pick(&context, &camera, position, &monkey) {
                        pick_mesh.set_transformation(Mat4::from_translation(pick));
                        change = true;
                    }
                }
            }
        }

        change |= control.handle_events(&mut camera, &mut frame_input.events);

        // draw
        if change {
            frame_input
                .screen()
                .clear(ClearState::color_and_depth(1.0, 1.0, 1.0, 1.0, 1.0))
                .render(
                    &camera,
                    &monkey,
                    &[&light, &ambient],
                    // &[&ambient, &directional],
                );
        }

        FrameOutput {
            swap_buffers: change,
            ..Default::default()
        }
    });
}
