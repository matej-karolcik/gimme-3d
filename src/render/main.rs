use std::path::Path;

use three_d::*;

#[tokio::main]
async fn main() {
    let context = HeadlessContext::new().unwrap();
    let _ = std::fs::create_dir("results");

    run("glb/1_p1_duvet-cover_1350x2000.glb", &context).await;
    return;


    let dirs = std::fs::read_dir("glb").unwrap();
    for dir in dirs {
        let dir = dir.unwrap();
        let path = dir.path();
        run(path.to_str().unwrap(), &context).await;
    }
}

async fn run(model_path: &str, context: &HeadlessContext) {
    let start = std::time::Instant::now();

    println!("Running: {}", model_path);

    let width = 2000;
    let height = 2000;

    let maybe_pixels = rs3d::render::render(
        model_path,
        "https://www.w3.org/MarkUp/Test/xhtml-print/20050519/tests/jpeg420exif.jpg",
        &context,
        width,
        height,
    ).await;

    if maybe_pixels.is_err() {
        println!("Failed to render: {}", maybe_pixels.err().unwrap());
        return;
    }

    let pixels = maybe_pixels.unwrap();

    let img = image::load_from_memory(&pixels).unwrap();
    let mut writer = std::fs::File::create(Path::new("results")
        .join(Path::new(&model_path).file_name().unwrap())
        .with_extension("webp")).unwrap();

    img.write_to(&mut writer, image::ImageOutputFormat::WebP).unwrap();

    println!("Time: {:?}", std::time::Instant::now() - start);
}
