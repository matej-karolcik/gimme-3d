use anyhow::{Context, Result};
use image::{DynamicImage, ImageBuffer, Rgba};
use log::info;
use nalgebra::Point3;
use three_d::{
    vec3, Blend, Camera, ClearState, ColorMaterial, CpuTexture, Cull, DepthTexture2D, Model,
    RenderTarget, Texture2D, Texture2DRef,
};
use three_d_asset::io::Deserialize;
use three_d_asset::{radians, Interpolation, Viewport, Wrapping};

use crate::error::Error;
use crate::{img, model};

pub async fn render_urls(
    remote_model_path: Option<String>,
    model_bytes: Option<Vec<u8>>,
    textures: Vec<String>,
    context: &three_d::Context,
    width: u32,
    height: u32,
    local_model_dir: &String,
) -> Result<DynamicImage> {
    let texture_futures = textures
        .iter()
        .map(|url| tokio::spawn(img::download_img(url.clone())));

    let start = std::time::Instant::now();

    let (mut loaded_assets, final_model_path) =
        model::load(remote_model_path, local_model_dir, model_bytes).await?;
    let model_vec = Vec::from(
        loaded_assets
            .get(final_model_path.clone().as_str())
            .map_err(Error::AssetLoadingError)?,
    );

    let gltf = gltf::Gltf::from_slice(model_vec.as_slice()).map_err(Error::GltfParsingError)?;
    let doc = gltf.document;

    let model =
        three_d_asset::Model::deserialize(final_model_path.clone().as_str(), &mut loaded_assets)
            .context("loading model")?;

    info!("Model load: {:?}", std::time::Instant::now() - start);
    let start = std::time::Instant::now();

    let cpu_textures: Vec<_> = futures_util::future::join_all(texture_futures)
        .await
        .into_iter()
        .map(|result| {
            let mut cpu_texture = result.unwrap().unwrap();
            cpu_texture.data.to_linear_srgb();
            cpu_texture
        })
        .collect();

    info!("Textures load: {:?}", std::time::Instant::now() - start);

    render(context, model, cpu_textures, doc, width, height)
}

pub async fn render_raw_images(
    model_path: Option<String>,
    model_bytes: Option<Vec<u8>>,
    raw_textures: Vec<Vec<u8>>,
    context: &three_d::Context,
    width: u32,
    height: u32,
    local_model_path: &String,
) -> Result<DynamicImage> {
    let start = std::time::Instant::now();

    let cpu_textures = raw_textures
        .iter()
        .map(|raw_texture| {
            let mut cpu_texture = img::decode_img(raw_texture.as_slice()).expect("decoding image");
            cpu_texture.data.to_linear_srgb();
            cpu_texture
        })
        .collect();

    info!("Textures load: {:?}", std::time::Instant::now() - start);
    let start = std::time::Instant::now();

    let (mut loaded_assets, final_model_path) =
        model::load(model_path, local_model_path, model_bytes).await?;

    let model_vec = Vec::from(
        loaded_assets
            .get(final_model_path.clone())
            .map_err(Error::AssetLoadingError)?,
    );

    let gltf = gltf::Gltf::from_slice(model_vec.as_slice()).map_err(Error::GltfParsingError)?;
    let doc = gltf.document;

    let model = loaded_assets.deserialize(final_model_path.as_str())?;

    info!("Model load: {:?}", std::time::Instant::now() - start);

    render(context, model, cpu_textures, doc, width, height)
}

fn render(
    context: &three_d::Context,
    model: three_d_asset::Model,
    cpu_textures: Vec<CpuTexture>,
    doc: gltf::Document,
    width: u32,
    height: u32,
) -> Result<DynamicImage> {
    let start = std::time::Instant::now();

    let scene = doc.default_scene().ok_or(Error::NoDefaultScene)?;
    let camera_props =
        crate::gltf::extract(&scene, crate::gltf::get_camera).ok_or(Error::NoCamera)?;
    let mesh_props = crate::gltf::extract_all(&scene, crate::gltf::get_mesh);

    if mesh_props.is_empty() {
        return Err(Error::NoMesh.into());
    }

    let mut mesh = Model::<ColorMaterial>::new(context, &model).context("creating mesh")?;
    let num_textures = cpu_textures.len();

    let mut textures: Vec<&three_d_asset::Texture2D> = vec![];
    for cpu_texture in cpu_textures.iter() {
        let texture = &cpu_texture;
        textures.push(texture);
    }

    mesh.iter_mut().enumerate().for_each(|(pos, m)| {
        m.material.texture = Some(Texture2DRef::from_cpu_texture(
            context,
            &cpu_textures[pos % num_textures],
        ));
        m.material.is_transparent = true;
        m.material.render_states.cull = Cull::None;
        m.material.render_states.blend = Blend::STANDARD_TRANSPARENCY;
    });

    let camera_transform = camera_props.parent_transform * camera_props.transform;
    let point = camera_transform.position();

    let camera_rotation = camera_transform.rotation();
    let at = camera_rotation.transform_point(&Point3::new(0.0, 0.0, -1.0));
    let up = camera_rotation.transform_point(&Point3::new(0.0, 1.0, 0.0));

    let viewport = Viewport::new_at_origo(width, height);
    const FACTOR: f32 = 100.;

    let yfov = camera_props.yfov * (camera_props.aspect_ratio / (width as f32 / height as f32));

    let camera = Camera::new_perspective(
        viewport,
        vec3(point.x, point.y, point.z),
        vec3(at.x, at.y, at.z),
        vec3(up.x, up.y, up.z),
        radians(yfov),
        camera_props.znear / FACTOR,
        camera_props.zfar * FACTOR,
    );

    let mut texture = Texture2D::new_empty::<[u8; 4]>(
        context,
        viewport.width,
        viewport.height,
        Interpolation::Nearest,
        Interpolation::Nearest,
        None,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );

    let mut depth_texture = DepthTexture2D::new::<f32>(
        context,
        viewport.width,
        viewport.height,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );

    let pixels: Vec<[u8; 4]> = RenderTarget::new(
        texture.as_color_target(None),
        depth_texture.as_depth_target(),
    )
    .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
    .render(&camera, &mesh, &[])
    .read_color();

    let img = DynamicImage::ImageRgba8(
        ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
            viewport.width,
            viewport.height,
            pixels.iter().flat_map(|v| *v).collect::<Vec<_>>(),
        )
        .unwrap(),
    );

    info!("Time render: {:?}", std::time::Instant::now() - start);

    Ok(img)
}
