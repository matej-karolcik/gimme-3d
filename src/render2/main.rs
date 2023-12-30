use std::path::Path;

use three_d::*;

#[tokio::main]
async fn main() {
    let context = HeadlessContext::new().unwrap();
    // run("output/2_p1_hoodie_out/2_p1_hoodie.gltf", &context).await;
    // run("output/NotebookA5_out/NotebookA5.gltf", &context).await;
    // run("output/PhoneCase_IPhone12_out/PhoneCase_IPhone12.gltf", &context).await;
    // run("output/3_p1_shower-curtain_1800x2000_out/3_p1_shower-curtain_1800x2000.gltf", &context).await;
    // run("output/1_p1_hoodie_out/1_p1_hoodie.gltf", &context).await;
    // run("output/2_p1_sweater_out/2_p1_sweater.gltf", &context).await;
    // run("output/1_p1_t-shirt_out/1_p1_t-shirt.gltf", &context).await;
    // run("output/0_p3_bath-towel_out/0_p3_bath-towel.gltf", &context).await;
    // run("output/0_p3_bath-towel.glb", &context).await;
    // run("https://jq-staging-matko.s3.eu-central-1.amazonaws.com/gltf/PhoneCase_IPhone12.glb", &context).await;
    // return;

    let _ = std::fs::create_dir("results");
    let dirs = std::fs::read_dir("glb").unwrap();
    for dir in dirs {
        let dir = dir.unwrap();
        let path = dir.path();
        run(path.to_str().unwrap(), &context).await;
    }
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

    let width = 1000;
    let height = 1000;

    let maybe_pixels = gimme_the_3d::render::render(
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
