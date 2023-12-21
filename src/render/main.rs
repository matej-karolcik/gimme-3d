use std::path::Path;

use three_d::*;
use three_d_asset::io::Serialize;

use gimme_the_3d::camera;
use gimme_the_3d::mesh;

#[tokio::main]
async fn main() {
    let context = HeadlessContext::new().unwrap();
    // run("output/2_p1_hoodie_out/2_p1_hoodie.gltf", &context).await;
    // run("output/NotebookA5_out/NotebookA5.gltf", &context).await;
    // run("output/PhoneCase_IPhone12_out/PhoneCase_IPhone12.gltf", &context).await;
    // run("output/3_p1_shower-curtain_1800x2000_out/3_p1_shower-curtain_1800x2000.gltf", &context).await;
    // run("output/hoodie_out/hoodie.gltf", &context).await;
    // run("output/2_p1_sweater_out/2_p1_sweater.gltf", &context).await;
    run("output/1_p1_t-shirt_out/1_p1_t-shirt.gltf", &context).await;
    // run("output/0_p3_bath-towel_out/0_p3_bath-towel.gltf", &context).await;
    return;

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

    let camera_props = camera::extract_camera(&scene).unwrap();
    let mesh_props = mesh::extract_mesh(&scene).unwrap();

    let mut loaded = three_d_asset::io::load_async(&[
        "test2.png",
        model_path,
    ]).await.unwrap();

    let mut cpu_texture: CpuTexture = loaded.deserialize("test2.png").unwrap();
    cpu_texture.data.to_linear_srgb();

    let model = loaded.deserialize(model_path).unwrap();

    let mut global_transform: Option<gimme_the_3d::object::Transform> = None;
    let mut mesh_rot: Option<[f32; 4]> = None;

    let mut mesh = Model::<ColorMaterial>::new(&context, &model).unwrap();
    mesh.iter_mut().for_each(|m| {
        let _global_transform = m.transformation();
        let glob_transform = gimme_the_3d::object::Transform::from(_global_transform);

        // todo deduplicate rotation
        println!("global {:?}", glob_transform.decomposed());
        println!("local {:?}", mesh_props.transform.decomposed());
        let final_mesh_transform = (glob_transform * mesh_props.transform).decomposed();
        println!("global*local {:?}", final_mesh_transform);
        println!("camera {:?}", camera_props.transform.decomposed());

        let (_, rotation, _) = mesh_props.transform.decomposed();
        // if glob_transform != mesh_props.transform {
        //     mesh_rot = Some(rotation);
        // }
        let (translation, rotation, scale) = glob_transform.decomposed();
        let mut matrix_rotation = nalgebra::Quaternion::new(
            rotation[3],
            rotation[0],
            rotation[1],
            rotation[2],
        );

        let new_global = gimme_the_3d::object::Transform::from(
            gltf::scene::Transform::Decomposed {
                translation,
                scale,
                rotation: [
                    matrix_rotation.coords.x,
                    matrix_rotation.coords.y,
                    matrix_rotation.coords.z,
                    matrix_rotation.coords.w
                ],
            }
        );

        global_transform = Some(glob_transform);

        let foobar: Mat4 = glob_transform.into();

        m.set_transformation(foobar.concat(&mesh_props.transform.into()));

        m.material.texture = Some(Texture2DRef::from_cpu_texture(&context, &cpu_texture));
        m.material.is_transparent = false;
        m.material.render_states.cull = Cull::Back;
        m.material.render_states.cull = Cull::None;
        m.material.render_states.blend = Blend::STANDARD_TRANSPARENCY;
    });

    let mut glob_transform = global_transform.unwrap();
    if mesh_rot.is_some() {
        let mesh_rot = mesh_rot.unwrap();
        let mesh_transform = gimme_the_3d::object::Transform::from_quaternion(
            nalgebra::Quaternion::new(
                mesh_rot[3],
                mesh_rot[0],
                mesh_rot[1],
                mesh_rot[2],
            )
        );
        // glob_transform = glob_transform * mesh_transform;
    }

    let camera_transform = glob_transform * camera_props.transform;
    let origin = nalgebra::Point3::origin();
    let point = camera_transform.matrix.transform_point(&origin);
    let at = glob_transform.matrix.transform_point(&origin);

    let camera = Camera::new_perspective(
        viewport,
        vec3(point.x, point.y, point.z),
        vec3(at.x, at.y, at.z),
        vec3(0.0, 1.0, 0.0),
        radians(camera_props.yfov),
        0.01,
        camera_props.zfar,
    );

    // Create a color texture to render into
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

    // Also create a depth texture to support depth testing
    let mut depth_texture = DepthTexture2D::new::<f32>(
        &context,
        viewport.width,
        viewport.height,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );

    // Create a render target (a combination of a color and a depth texture) to write into
    let pixels = RenderTarget::new(
        texture.as_color_target(None),
        depth_texture.as_depth_target(),
    )
        // Clear color and depth of the render target
        // .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0))
        // Render the triangle with the per vertex colors defined at construction
        .render(&camera, &mesh, &[])
        // Read out the colors from the render target
        .read_color();

    let result_path = Path::new("results")
        .join(Path::new(&model_path).file_name().unwrap())
        .with_extension("png");

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
