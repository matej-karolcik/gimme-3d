use std::path::Path;

use three_d::*;

pub async fn run_multiple(input: &String, results: &String, context: &HeadlessContext) {
    let files = std::fs::read_dir(input).unwrap();
    for file in files {
        let entry = file.unwrap();
        let path = entry.path();
        run(path.to_str().unwrap(), results, &context).await;
    }
}

async fn run(model_path: &str, results_path: &String, context: &HeadlessContext) {
    let start = std::time::Instant::now();

    println!("Running: {}", model_path);

    let width = 2000;
    let height = 2000;

    let textures = vec![
        String::from("https://www.w3.org/MarkUp/Test/xhtml-print/20050519/tests/jpeg420exif.jpg"),
    ];

    let maybe_pixels = crate::render::render_urls(
        String::from(model_path),
        textures,
        &context,
        width,
        height,
        &String::new(),
    ).await;

    if maybe_pixels.is_err() {
        println!("Failed to render: {}", maybe_pixels.err().unwrap());
        return;
    }

    let pixels = maybe_pixels.unwrap();

    let img = image::load_from_memory(&pixels).unwrap();
    let mut writer = std::fs::File::create(Path::new(results_path)
        .join(Path::new(&model_path).file_name().unwrap())
        .with_extension("webp")).unwrap();

    img.write_to(&mut writer, image::ImageOutputFormat::WebP).unwrap();

    println!("Time: {:?}", std::time::Instant::now() - start);
}
