use std::path::{Path, PathBuf};

use anyhow::anyhow;
use anyhow::Result;
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use three_d::*;

#[tokio::main]
async fn main() {
    let mask_files = walk_directory("masks".to_string());

    let canvas = "testdata/canvas.png".to_string();

    let context = HeadlessContext::new().unwrap();
    let _ = std::fs::create_dir("results");
    let _ = std::fs::create_dir("textures");

    // run(
    //     &context,
    //     "masks/s1_p1_hoodie/00_s1_p1_hoodie.webp".to_string(),
    //     canvas.to_string(),
    // ).await.unwrap();
    // run(
    //     &context,
    //     "masks/s1_p1_t-shirt/00_s1_p1_t-shirt.webp".to_string(),
    //     canvas.to_string(),
    // ).await.unwrap();
    // run(
    //     &context,
    //     "masks/s1_p1_notebook/00_s1_p1_notebook_a5.webp".to_string(),
    //     canvas.to_string(),
    // ).await.unwrap();
    // run(
    //     &context,
    //     "masks/s0_p3_towel-bath/00_s0_p3_towel-bath.png".to_string(),
    //     canvas.to_string(),
    // ).await.unwrap();
    // run(
    //     &context,
    //     "masks/s1_p1_set-pillowcase-duvet-cover/00_s1_p1_set-pillowcase-duvet-cover.webp".to_string(),
    //     canvas.to_string(),
    // ).await.unwrap();
    run(
        &context,
        "masks/s3_p1_sweatshirt/00_s3_p1_sweatshirt.webp".to_string(),
        canvas.to_string(),
    ).await.unwrap();
    return;

    for mask in mask_files {
        let result = run(&context, mask.clone(), canvas.to_string()).await;
        if let Err(e) = result {
            println!("Error in {}: {}", mask, e);
        }
    }
}

fn walk_directory(dir: String) -> Vec<String> {
    let mut result = vec![];

    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            result.append(&mut walk_directory(path.to_str().unwrap().to_string()));
        } else {
            result.push(path.to_str().unwrap().to_string());
        }
    }

    result
}

async fn run(context: &HeadlessContext, mask: String, canvas: String) -> Result<()> {
    let start = std::time::Instant::now();

    let model_file = PathBuf::from(&mask)
        .with_extension("glb")
        .file_name()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap();

    if !model_file.starts_with("00_s") {
        return Err(anyhow!(format!(
            "model file name must start with 00_s: {}",
            model_file
        )));
    }

    let model_file = model_file.strip_prefix("00_s").unwrap().to_string();

    let mask = image::open(mask).unwrap();
    let texture_bytes = std::fs::read(canvas).unwrap();

    const UPSCALE: u32 = 2;

    let pixels = gimme_3d::render::render_raw_images(
        Some(
            Path::new("glb")
                .join(model_file.clone())
                .to_str()
                .unwrap()
                .to_string(),
        ),
        None,
        vec![texture_bytes],
        context,
        mask.width() * UPSCALE,
        mask.height() * UPSCALE,
        &String::new(),
    )
    .await?;

    let texture: DynamicImage = image::imageops::resize(
        &pixels,
        mask.width(),
        mask.height(),
        image::imageops::FilterType::Triangle,
    )
    .into();
    // texture.save(Path::new("textures").join(Path::new(&model_file).with_extension("png")))?;

    let result = multiply(&mask, &texture);

    let mut writer = std::fs::File::create(
        Path::new("results")
            .join(Path::new(&model_file).file_name().unwrap())
            .with_extension("webp"),
    )?;

    result.write_to(&mut writer, image::ImageOutputFormat::WebP)?;

    println!("{:<width$}{:?}", model_file, start.elapsed(), width = 50);

    Ok(())
}

/// images should have the same dimensions
fn multiply(bottom: &DynamicImage, top_raw: &DynamicImage) -> DynamicImage {
    let top;
    if top_raw.dimensions() != bottom.dimensions() {
        top = top_raw.resize(
            bottom.width(),
            bottom.height(),
            image::imageops::FilterType::Lanczos3,
        );
    } else {
        top = top_raw.clone();
    }

    let mut result_image = RgbaImage::new(bottom.width(), bottom.height());

    for y in 0..bottom.height() {
        for x in 0..bottom.width() {
            let pixel1 = bottom.get_pixel(x, y);
            let pixel2 = top.get_pixel(x, y);

            let mut result = vec![];
            (0..3).for_each(|i| {
                let ch1 = pixel1[i] as f32 / 255.;
                let ch2 = pixel2[i] as f32 / 255.;

                if ch1 == 0. || ch2 == 0. {
                    result.push((ch1.max(ch2) * 255.) as u8);
                    return;
                }

                result.push((ch1 * ch2 * 255.) as u8);
            });

            result.push(pixel1[3]);

            let product: [u8; 4] = result.try_into().unwrap();
            let product = Rgba(product);

            result_image.put_pixel(x, y, product);
        }
    }

    result_image.into()
}
