use anyhow::{anyhow, Result};
use three_d::{Blend, Camera, ClearState, ColorMaterial, CpuTexture, Cull, DepthTexture2D, Model, RenderTarget, Texture2D, Texture2DRef, vec3};
use three_d_asset::{Interpolation, radians, Viewport, Wrapping};

type RawPixels = Vec<[u8; 4]>;

pub async fn render(
    model_path: &str,
    texture_path: &str,
    context: &three_d::Context,
    width: u32,
    height: u32,
) -> Result<RawPixels> {
    let to_load = &[texture_path, model_path];
    let loaded_future = three_d_asset::io::load_async(to_load);

    let (doc, _, _) = gltf::import(model_path)
        .map_err(|e| anyhow!("Error loading gltf: {:?}", e))?;

    let scene = doc.default_scene()
        .ok_or(anyhow!("No default scene"))?;
    let camera_props = crate::gltf::extract(&scene, crate::gltf::get_camera)
        .ok_or(anyhow!("No camera found"))?;
    let mesh_props = crate::gltf::extract(&scene, crate::gltf::get_mesh)
        .ok_or(anyhow!("No mesh found"))?;

    let mut loaded_assets = loaded_future.await
        .map_err(|e| anyhow!("Error loading assets: {:?}", e))?;

    let mut cpu_texture: CpuTexture = loaded_assets.deserialize(texture_path)
        .map_err(|e| anyhow!("Error loading texture: {:?}", e))?;
    cpu_texture.data.to_linear_srgb();

    let model = loaded_assets.deserialize(model_path)
        .map_err(|e| anyhow!("Error loading model: {:?}", e))?;

    let mut mesh = Model::<ColorMaterial>::new(&context, &model)
        .map_err(|e| anyhow!("Error loading model: {:?}", e))?;

    mesh.iter_mut().for_each(|m| {
        let final_transform = mesh_props.parent_transform * mesh_props.transform;
        m.set_transformation(final_transform.into());

        m.material.texture = Some(Texture2DRef::from_cpu_texture(&context, &cpu_texture));
        m.material.is_transparent = true;
        // todo do we need this?
        m.material.render_states.cull = Cull::Back;
        m.material.render_states.blend = Blend::STANDARD_TRANSPARENCY;
    });

    let camera_transform = camera_props.parent_transform * camera_props.transform;
    let origin = nalgebra::Point3::origin();
    let point = camera_transform.matrix.transform_point(&origin);
    let at = camera_props.parent_transform.matrix.transform_point(&origin);

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

    // todo rework this
    if (camera_props.parent_transform.decomposed().1[0].abs() - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.0001
        && camera_props.transform.has_equal_rotation(&crate::object::Transform::from_quaternion(nalgebra::Quaternion::identity())) {
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

    let result = RenderTarget::new(
        texture.as_color_target(None),
        depth_texture.as_depth_target(),
    )
        .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
        .render(&camera, &mesh, &[])
        .read_color();

    Ok(result)
}
