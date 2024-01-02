use std::path::Path;

use three_d::*;

#[tokio::main]
async fn main() {
    let context = HeadlessContext::new().unwrap();
    let _ = std::fs::create_dir("results");
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

    let width = 800;
    let height = 800;

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

    let result_path = Path::new("results")
        .join(Path::new(&model_path).file_name().unwrap())
        .with_extension("png");

    std::fs::write(&result_path, pixels).unwrap();

    println!("Time serialize: {:?}", std::time::Instant::now() - start);
}
