use three_d::*;

use gimme_3d::render_file::run_multiple;

#[tokio::main]
async fn main() {
    let context = HeadlessContext::new().unwrap();
    let _ = std::fs::create_dir("results");

    // run(
    //     "glb/0_p3_bath-towel.glb",
    //     &String::from("results"),
    //     &context,
    //     &None,
    // ).await;
    // gimme_3d::render_file::run(
    //     "glb/1_p1_t-shirt.glb",
    //     &String::from("results"),
    //     &context,
    //     &Some(&String::from("testdata/canvas.png")),
    // ).await;
    gimme_3d::render_file::run(
        "glb/sweatshirt.glb",
        &String::from("results"),
        &context,
        &None,
    )
    .await;
    return;

    let dirs = std::fs::read_dir("glb").unwrap();
    for dir in dirs {
        let dir = dir.unwrap();
        let path = dir.path();

        run_multiple(
            &String::from(path.to_str().unwrap()),
            &String::from("results"),
            &context,
            &None,
        )
        .await;
    }
}
