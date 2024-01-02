use anyhow::Result;
use env_logger::Target;
use serde::{Deserialize, Serialize};
use three_d::HeadlessContext;
use tokio::sync::mpsc;
use warp::Filter;
use warp::reply::Response;

use rs3d::render::RawPixels;

#[derive(Deserialize, Serialize)]
struct Request {
    model_path: String,
    texture_path: String,
    width: u32,
    height: u32,
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .target(Target::Stdout)
        .init();

    let context = HeadlessContext::new().unwrap();

    let (request_tx, mut request_rx) = mpsc::channel::<(Request, mpsc::Sender<Result<RawPixels>>)>(1);

    tokio::spawn(async move { serve(request_tx).await; });

    loop {
        let (request, response_tx) = request_rx.recv().await.unwrap();
        let pixels = rs3d::render::render(
            request.model_path.as_str(),
            request.texture_path.as_str(),
            &context,
            request.width,
            request.height,
        ).await;
        response_tx.send(pixels).await.unwrap();
    }
}


async fn serve(request_tx: mpsc::Sender<(Request, mpsc::Sender<Result<RawPixels>>)>) {
    let render = warp::post()
        .and(warp::path("render"))
        .and(warp::body::json())
        .and_then(move |r: Request| {
            let request_tx = request_tx.clone();
            async move {
                let (response_tx, mut response_rx) = mpsc::channel(1);
                request_tx.try_send((r, response_tx)).unwrap();
                let r = response_rx.recv().await.unwrap().unwrap();

                // this is fast only on linux for some reason
                // let start = std::time::Instant::now();
                // let img = image::load_from_memory(&r).unwrap();
                // let mut writer = std::io::Cursor::new(Vec::new());
                // img.write_to(&mut writer, image::ImageOutputFormat::WebP).unwrap();
                // log::info!("Converted to webp in: {:?}", start.elapsed());

                Ok::<Response, warp::Rejection>(warp::http::response::Builder::new()
                    // .header("Content-Type", "image/webp")
                    .header("Content-Type", "image/png")
                    // .body(writer.into_inner().into())
                    .body(r.into())
                    .unwrap())
            }
        });

    let health = warp::get()
        .and(warp::path("health"))
        .map(|| "ok");

    let routes = render.or(health);

    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}
