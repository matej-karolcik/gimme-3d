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
    run("output/hoodie_out/hoodie.gltf", &context).await;
    // run("output/PhoneCase_IPhone12_out/PhoneCase_IPhone12.gltf", &context).await;
    return;

    let dirs = std::fs::read_dir("output").unwrap();
    for dir in dirs {
        let dir = dir.unwrap();
        let path = dir.path();
        if path.is_dir() {
            let files = std::fs::read_dir(path).unwrap();
            for file in files {
                let file = file.unwrap();
                let path = file.path();
                println!("Path: {:?}", path);
                if path.is_file() && path.to_str().unwrap().ends_with(".gltf") {
                    println!("Running: {}", path.to_str().unwrap());
                    run(path.to_str().unwrap(), &context).await;
                }
            }
        }
    }
}

async fn run(model_path: &str, context: &HeadlessContext) {
    let start = std::time::Instant::now();
    println!("Loading: {}", model_path);

    let viewport = Viewport::new_at_origo(889, 800);
    // let context = HeadlessContext::new().unwrap();

    let (doc, _, _) = gltf::import(model_path).unwrap();

    let default_scene_maybe = doc.default_scene();

    if let None = default_scene_maybe {
        panic!("No default scene");
    }

    let scene = default_scene_maybe.unwrap();

    let camera_props = camera::extract_camera(&scene).unwrap();
    let mesh_props = mesh::extract_mesh(&scene).unwrap();

    let point = camera_props.rotation.transform_point(&camera_props.position);
    let point = camera_props.scale.transform_point(&point);

    let camera = Camera::new_perspective(
        viewport,
        vec3(point.x, point.y, point.z),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        radians(camera_props.yfov),
        0.01,
        camera_props.zfar,
    );

    let mut loaded = three_d_asset::io::load_async(&[
        "test2.png",
        model_path,
    ]).await.unwrap();

    let mut cpu_texture: CpuTexture = loaded.deserialize("test2.png").unwrap();
    cpu_texture.data.to_linear_srgb();

    let model = loaded.deserialize(model_path).unwrap();

    println!("loaded stuff: {:?}", std::time::Instant::now() - start);

    let mut mesh = Model::<ColorMaterial>::new(&context, &model).unwrap();
    mesh.iter_mut().for_each(|m| {
        let point = mesh_props.rotation.transform_point(&mesh_props.position);
        let point = mesh_props.scale.transform_point(&point);

        m.set_transformation(
            Mat4::from_translation(vec3(
                point.x,
                point.y,
                point.z,
            ))
        );

        m.material.texture = Some(Texture2DRef::from_cpu_texture(&context, &cpu_texture));
        m.material.is_transparent = false;
        m.material.render_states.cull = Cull::Back;
        m.material.render_states.blend = Blend::STANDARD_TRANSPARENCY;
    });

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

    let result_path = Path::new(&model_path);

    three_d_asset::io::save(
        &CpuTexture {
            data: TextureData::RgbaU8(pixels),
            width: texture.width(),
            height: texture.height(),
            ..Default::default()
        }
            .serialize(result_path.with_extension("png").file_name().unwrap())
            .unwrap(),
    )
        .unwrap();

    println!("Time: {:?}", std::time::Instant::now() - start);
}
