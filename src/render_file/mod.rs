use std::path::Path;

use image::DynamicImage;
use three_d::*;

pub async fn run_multiple(
    input: &String,
    results: &String,
    context: &HeadlessContext,
    texture_url: &Option<&String>,
) {
    let files = std::fs::read_dir(input).unwrap();
    for file in files {
        let entry = file.unwrap();
        let path = entry.path();
        run(path.to_str().unwrap(), results, &context, texture_url).await;
    }
}

pub async fn run(
    model_path: &str,
    results_path: &String,
    context: &HeadlessContext,
    texture_url: &Option<&String>,
) {
    let start = std::time::Instant::now();

    println!("Running: {}", model_path);

    let factor = 2;
    let width = 2222;
    let height = 2000;

    let texture = if let Some(texture_url) = texture_url {
        let texture_url = *texture_url;
        texture_url.to_owned()
    } else {
        String::from("https://www.shutterstock.com/shutterstock/photos/72627163/display_1500/stock-vector-color-test-for-television-for-checking-quality-also-available-as-jpeg-72627163.jpg")
    };

    let textures = vec![texture];

    let maybe_pixels = crate::render::render_urls(
        Some(String::from(model_path)),
        None,
        textures,
        &context,
        width * factor,
        height * factor,
        &String::new(),
    ).await;

    if maybe_pixels.is_err() {
        println!("Failed to render: {}", maybe_pixels.err().unwrap());
        return;
    }

    let pixels = maybe_pixels.unwrap();

    let img: DynamicImage = image::imageops::resize(
        &pixels,
        width,
        height,
        image::imageops::FilterType::Lanczos3,
    ).into();

    let mut writer = std::fs::File::create(Path::new(results_path)
        .join(Path::new(&model_path).file_name().unwrap())
        .with_extension("webp")).unwrap();

    img.write_to(&mut writer, image::ImageOutputFormat::WebP).unwrap();

    println!("Time: {:?}", std::time::Instant::now() - start);
}
