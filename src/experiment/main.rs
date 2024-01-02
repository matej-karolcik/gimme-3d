use three_d::HeadlessContext;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = tokio::sync::oneshot::channel::<Message>();

    tokio::spawn(async move {
        tx.send(Message {
            model_path: "output/2_p1_hoodie_out/2_p1_hoodie.gltf".to_string(),
            texture_path: "test2.png".to_string(),
            width: 889,
            height: 800,
        }).unwrap();
    });

    let context = HeadlessContext::new().unwrap();
    let message = rx.await.unwrap();
    let pixels = rs3d::render::render(
        message.model_path.as_str(),
        message.texture_path.as_str(),
        &context,
        message.width,
        message.height,
    ).await.unwrap();

    std::fs::write("channels.png", pixels).unwrap();
}

#[derive(Debug)]
struct Message {
    model_path: String,
    texture_path: String,
    width: u32,
    height: u32,
}
