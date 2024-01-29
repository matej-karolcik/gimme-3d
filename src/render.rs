use std::f32::consts::FRAC_1_SQRT_2;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use log::info;
use nalgebra::Quaternion;
use three_d::{Attenuation, Blend, Camera, ClearState, ColorMaterial, CpuTexture, Cull, DepthTexture2D, Model, PointLight, RenderTarget, Texture2D, Texture2DRef, vec3};
use three_d_asset::{Interpolation, radians, Srgba, TextureData, Viewport, Wrapping};
use three_d_asset::io::Serialize;

use crate::error::Error;
use crate::img;
use crate::object::{Light, LightKind, Transform};

pub type RawPixels = Vec<u8>;

pub async fn render_urls(
    model_path: String,
    textures: Vec<String>,
    context: &three_d::Context,
    width: u32,
    height: u32,
    local_model_dir: &String,
) -> Result<RawPixels> {
    let start = std::time::Instant::now();

    let mut to_load = textures.clone();
    let final_model_path;

    if let Ok(local_model_path) = get_local_model(local_model_dir, &model_path.clone()) {
        final_model_path = local_model_path.clone();
        to_load.push(local_model_path);
    } else {
        final_model_path = model_path.clone();
        to_load.push(model_path.clone());
    }

    let loaded_future = three_d_asset::io::load_async(to_load.as_slice());
    let mut loaded_assets = loaded_future.await.map_err(|e| Error::AssetLoadingError(e))?;

    info!("Assets load: {:?}", std::time::Instant::now() - start);
    let start = std::time::Instant::now();

    let model_slice = Vec::from(loaded_assets.get(final_model_path.clone().as_str())
        .map_err(|e| Error::AssetLoadingError(e))?);

    let gltf = gltf::Gltf::from_slice(model_slice.as_slice()).map_err(|e| Error::GltfParsingError(e))?;
    let doc = gltf.document;

    let model = loaded_assets.deserialize(final_model_path.clone().as_str()).context("loading model")?;

    info!("Model load: {:?}", std::time::Instant::now() - start);
    let start = std::time::Instant::now();

    let cpu_textures: Vec<CpuTexture> = textures.iter()
        .map(|texture_path| {
            let mut cpu_texture: CpuTexture = loaded_assets
                .deserialize(texture_path)
                .context("loading texture")
                .unwrap();
            cpu_texture.data.to_linear_srgb();
            cpu_texture
        })
        .collect();

    info!("Textures load: {:?}", std::time::Instant::now() - start);

    render(
        &context,
        model,
        cpu_textures,
        doc,
        width,
        height,
    )
}

pub async fn render_raw_images(
    model_path: String,
    raw_textures: Vec<Vec<u8>>,
    context: &three_d::Context,
    width: u32,
    height: u32,
    local_model_path: &String,
) -> Result<RawPixels> {
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

    let mut final_model_path = model_path.clone();
    let mut loaded_assets;

    if let Ok(model_path) = get_local_model(local_model_path, &model_path.clone()) {
        loaded_assets = three_d_asset::io::load(&[model_path.clone()]).unwrap();
        final_model_path = model_path.clone();
    } else {
        let to_load = vec![model_path.clone()];
        let loaded_future = three_d_asset::io::load_async(to_load.as_slice());

        loaded_assets = loaded_future.await.map_err(|e| Error::ModelNotFound(e))?;
    }

    let model_slice = Vec::from(loaded_assets.get(final_model_path.clone())
        .map_err(|e| Error::AssetLoadingError(e))?);

    let gltf = gltf::Gltf::from_slice(model_slice.as_slice()).map_err(|e| Error::GltfParsingError(e))?;
    let doc = gltf.document;

    let model = loaded_assets.deserialize(final_model_path.as_str()).context("loading model")?;

    info!("Model load: {:?}", std::time::Instant::now() - start);

    render(
        &context,
        model,
        cpu_textures,
        doc,
        width,
        height,
    )
}

fn get_local_model(local_dir: &String, path: &String) -> Result<String> {
    if local_dir.is_empty() || path.is_empty() {
        return Err(anyhow!("local directory or path is empty"));
    }

    let local_dir = Path::new(local_dir.as_str());
    let model_path = Path::new(path.as_str());
    let filename = model_path.file_name();

    if let None = filename {
        return Err(anyhow!("no filename found in {}", path));
    }

    let model_path = local_dir.join(filename.unwrap());

    if !model_path.exists() {
        return Err(Error::NoLocalModel(model_path.display().to_string()).into());
    }


    Ok(String::from(model_path.to_str().unwrap()))
}

fn render(
    context: &three_d::Context,
    model: three_d_asset::Model,
    cpu_textures: Vec<CpuTexture>,
    doc: gltf::Document,
    width: u32,
    height: u32,
) -> Result<RawPixels> {
    let start = std::time::Instant::now();

    let scene = doc.default_scene().ok_or(Error::NoDefaultScene)?;
    let camera_props = crate::gltf::extract(&scene, crate::gltf::get_camera).ok_or(Error::NoCamera)?;
    let mesh_props = crate::gltf::extract_all(&scene, crate::gltf::get_mesh);
    let light_props = crate::gltf::extract(&scene, crate::gltf::get_light);

    if mesh_props.is_empty() {
        return Err(Error::NoMesh.into());
    }

    let mut mesh = Model::<ColorMaterial>::new(&context, &model).context("creating mesh")?;
    let num_textures = cpu_textures.len();

    // let mut textures: Vec<&three_d_asset::Texture2D> = vec![];
    // for cpu_texture in cpu_textures.iter() {
    //     let texture = &cpu_texture;
    //     textures.push(texture);
    // }

    mesh.iter_mut()
        .enumerate()
        .for_each(|(pos, m)| {
            let mesh_props = mesh_props.get(pos).unwrap();
            let final_transform = mesh_props.parent_transform * mesh_props.transform;
            m.set_transformation(final_transform.into());

            m.material.texture = Some(Texture2DRef::from_cpu_texture(&context, &cpu_textures[pos % num_textures]));
            m.material.is_transparent = true;
            m.material.render_states.cull = Cull::None;
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
        radians(camera_props.yfov * camera_props.aspect_ratio),
        0.01,
        camera_props.zfar,
    );

    if (camera_props.parent_transform.decomposed().1[0].abs() - FRAC_1_SQRT_2).abs() < 0.0001
        && camera_props.transform.has_equal_rotation(&Transform::from_quaternion(Quaternion::identity())) {
        camera.roll(three_d_asset::Deg::<f32>(90.0));
    }

    let mut light: Option<Box<dyn three_d::Light>> = None;

    if let Some(light_props) = light_props {
        light = create_point_light(&light_props, &context);
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

    let lights = if let Some(light) = light.as_ref() {
        vec![light.as_ref()]
    } else {
        vec![]
    };

    // let effect = three_d::FxaaEffect {};
    // let render_depth_texture = DepthTexture2D::new::<f32>(
    //     &context,
    //     viewport.width,
    //     viewport.height,
    //     Wrapping::ClampToEdge,
    //     Wrapping::ClampToEdge,
    // );

    let pixels = RenderTarget::new(
        texture.as_color_target(None),
        depth_texture.as_depth_target(),
    )
        .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
        .render(&camera, &mesh, lights.as_slice())
        // .render_with_effect(
        //     &effect,
        //     &camera,
        //     &mesh,
        //     lights.as_slice(),
        //     Some(ColorTexture::Array {
        //         texture: &Texture2DArray::new(&context, textures.as_slice()),
        //         layers: &[textures.len() as u32],
        //     }),
        //     Some(DepthTexture::Single(&render_depth_texture)),
        // )
        .read_color();

    let result = CpuTexture {
        data: TextureData::RgbaU8(pixels.clone()),
        width: texture.width(),
        height: texture.height(),
        ..Default::default()
    }
        .serialize("result.png")
        .unwrap()
        .remove("result.png")
        .unwrap();

    info!("Time render: {:?}", std::time::Instant::now() - start);

    Ok(result)
}

fn create_point_light(
    light_props: &Light,
    context: &three_d::Context,
) -> Option<Box<dyn three_d::Light>> {
    let origin = nalgebra::Point3::origin();
    let light_transform = light_props.parent_transform * light_props.transform;
    let light_position = light_transform.matrix.transform_point(&origin);
    let light_color = Srgba::from(light_props.color);

    match light_props.kind {
        LightKind::Point => {
            Some(Box::new(PointLight::new(
                &context,
                light_props.intensity,
                light_color,
                &vec3(light_position.x, light_position.y, light_position.z),
                Attenuation::default(),
            )))
        }
        _ => { None }
    }
}
