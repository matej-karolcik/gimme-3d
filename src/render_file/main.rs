use three_d::*;

use rs3d::render_file::run_multiple;

#[tokio::main]
async fn main() {
    let context = HeadlessContext::new().unwrap();
    let _ = std::fs::create_dir("results");

    // run("glb/1_p1_duvet-cover_1350x2000.glb", &context).await;
    // run("glb/1_p1_t-shirt.glb", &context).await;
    // run("glb/PhoneCase_IPhone12.glb", &context).await;
    // run("output/PhoneCase_IPhone12_out/PhoneCase_IPhone12.gltf", &context).await;
    // return;


    let dirs = std::fs::read_dir("glb").unwrap();
    for dir in dirs {
        let dir = dir.unwrap();
        let path = dir.path();

        run_multiple(
            &String::from(path.to_str().unwrap()),
            &String::from("results"),
            &context,
            &None,
        ).await;
    }
}
