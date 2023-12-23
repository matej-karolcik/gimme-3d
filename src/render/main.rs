use std::path::Path;

use three_d::*;
use three_d_asset::io::Serialize;

#[tokio::main]
async fn main() {
    let context = HeadlessContext::new().unwrap();
    // run("output/2_p1_hoodie_out/2_p1_hoodie.gltf", &context).await;
    // run("output/NotebookA5_out/NotebookA5.gltf", &context).await;
    // run("output/PhoneCase_IPhone12_out/PhoneCase_IPhone12.gltf", &context).await;
    // run("output/3_p1_shower-curtain_1800x2000_out/3_p1_shower-curtain_1800x2000.gltf", &context).await;
    // run("output/1_p1_hoodie_out/1_p1_hoodie.gltf", &context).await;
    // run("output/2_p1_sweater_out/2_p1_sweater.gltf", &context).await;
    // run("output/1_p1_t-shirt_out/1_p1_t-shirt.gltf", &context).await;
    // run("output/0_p3_bath-towel_out/0_p3_bath-towel.gltf", &context).await;
    // return;

    let _ = std::fs::create_dir("results");
    let dirs = std::fs::read_dir("output").unwrap();
    for dir in dirs {
        let dir = dir.unwrap();
        let path = dir.path();
        if path.is_dir() {
            let files = std::fs::read_dir(path).unwrap();
            for file in files {
                let file = file.unwrap();
                let path = file.path();
                if path.is_file() && path.to_str().unwrap().ends_with(".gltf") {
                    run(path.to_str().unwrap(), &context).await;
                }
            }
        }
    }
}

async fn run(model_path: &str, context: &HeadlessContext) {
    let start = std::time::Instant::now();

    println!("Running: {}", model_path);

    let viewport = Viewport::new_at_origo(889, 800);

    let (doc, _, _) = gltf::import(model_path).unwrap();

    let default_scene_maybe = doc.default_scene();

    if let None = default_scene_maybe {
        panic!("No default scene");
    }

    let scene = default_scene_maybe.unwrap();

    let camera_props = gimme_the_3d::gltf::extract(&scene, gimme_the_3d::gltf::get_camera).unwrap();
    let mesh_props = gimme_the_3d::gltf::extract(&scene, gimme_the_3d::gltf::get_mesh).unwrap();

    let mut loaded = three_d_asset::io::load_async(&[
        "test2.png",
        model_path,
    ]).await.unwrap();

    let mut cpu_texture: CpuTexture = loaded.deserialize("test2.png").unwrap();
    cpu_texture.data.to_linear_srgb();

    let model = loaded.deserialize(model_path).unwrap();

    let mut mesh = Model::<ColorMaterial>::new(&context, &model).unwrap();
    mesh.iter_mut().for_each(|m| {
        // let _global_transform = m.transformation();
        // let mut glob_transform = gimme_the_3d::object::Transform::from(_global_transform);

        // println!("global {:?}", glob_transform.decomposed());
        // println!("parent {:?}", mesh_props.parent_transform.decomposed());
        // println!("mesh {:?}", mesh_props.transform.decomposed());

        // print_euler("global", &glob_transform);
        // print_euler("parent", &mesh_props.parent_transform);
        // print_euler("mesh", &mesh_props.transform);

        let final_transform = mesh_props.parent_transform * mesh_props.transform;

        m.set_transformation(final_transform.into());

        m.material.texture = Some(Texture2DRef::from_cpu_texture(&context, &cpu_texture));
        m.material.is_transparent = true;
        // m.material.render_states.cull = Cull::Back;
        m.material.render_states.cull = Cull::None;
        m.material.render_states.blend = Blend::STANDARD_TRANSPARENCY;
    });

    let camera_transform = camera_props.parent_transform * camera_props.transform;
    let origin = nalgebra::Point3::origin();
    let point = camera_transform.matrix.transform_point(&origin);
    let at = camera_props.parent_transform.matrix.transform_point(&origin);
    // println!("camera parent {:?}", camera_props.parent_transform.decomposed());
    // println!("camera {:?}", camera_props.transform.decomposed());
    // print_euler("camera parent", &camera_props.parent_transform);
    // print_euler("camera before", &camera_props.transform);
    // print_euler("camera after", &camera_transform);

    let width = 500;
    let height = width * camera_props.aspect_ratio as u32;
    let viewport = Viewport::new_at_origo(width, height);


    let mut camera = Camera::new_perspective(
        viewport,
        vec3(point.x, point.y, point.z),
        vec3(at.x, at.y, at.z),
        vec3(0.0, 1.0, 0.0),
        radians(camera_props.yfov),
        0.01,
        camera_props.zfar,
    );

    // todo this should be done properly
    // println!("camera {:?}", camera_props.parent_transform.decomposed().1[0].abs());
    if (camera_props.parent_transform.decomposed().1[0].abs() - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.0001
        && camera_props.transform.has_equal_rotation(&gimme_the_3d::object::Transform::from_quaternion(nalgebra::Quaternion::identity())) {
        camera.roll(three_d_asset::Deg::<f32>(90.0));
    }

    let mut texture = Texture2D::new_empty::<[u8; 4]>(
        &context,
        viewport.width,
        viewport.height,
        Interpolation::Nearest,
        Interpolation::Nearest,
        None,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );

    let mut depth_texture = DepthTexture2D::new::<f32>(
        &context,
        viewport.width,
        viewport.height,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );

    let pixels = RenderTarget::new(
        texture.as_color_target(None),
        depth_texture.as_depth_target(),
    )
        .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
        .render(&camera, &mesh, &[])
        .read_color();

    let result_path = Path::new("results")
        .join(Path::new(&model_path).file_name().unwrap())
        .with_extension("png");


    // raw bytes here
    // let foobar = CpuTexture {
    //     data: TextureData::RgbaU8(pixels.clone()),
    //     width: texture.width(),
    //     height: texture.height(),
    //     ..Default::default()
    // }.serialize("foobar.png").unwrap().remove("foobar.png").unwrap();
    //
    // std::fs::write("foobar.png", foobar).unwrap();

    three_d_asset::io::save(
        &CpuTexture {
            data: TextureData::RgbaU8(pixels),
            width: texture.width(),
            height: texture.height(),
            ..Default::default()
        }
            .serialize(result_path)
            .unwrap(),
    )
        .unwrap();

    println!("Time: {:?}", std::time::Instant::now() - start);
}

fn print_euler(label: &str, transform: &gimme_the_3d::object::Transform) {
    let (_, rot, _) = transform.decomposed();
    let rot = nalgebra::Rotation3::from(
        nalgebra::UnitQuaternion::from_quaternion(
            nalgebra::Quaternion::new(
                rot[3],
                rot[0],
                rot[1],
                rot[2],
            )
        )
    );

    let (x, y, z) = rot.euler_angles();
    println!("{} {:?}", label, [x.to_degrees(), y.to_degrees(), z.to_degrees()]);
}
