use three_d::*;

use rs3d::render_file::{run, run_multiple};

#[tokio::main]
async fn main() {
    let context = HeadlessContext::new().unwrap();
    let _ = std::fs::create_dir("results");

    run(
        "glb/0_p3_bath-towel.glb",
        &String::from("results"),
        &context,
        &None,
    ).await;
    // run(
    //     "glb/1_p1_t-shirt.glb",
    //     &String::from("results"),
    //     &context,
    //     &None,
    // ).await;
    // run(
    //     "glb/cushion002.glb",
    //     &String::from("results"),
    //     &context,
    //     &None,
    // ).await;
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
        ).await;
    }
}
