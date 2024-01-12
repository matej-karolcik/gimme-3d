use std::io::Write;
use std::sync::Arc;

use anyhow::Result;
use bytes::BufMut;
use env_logger::Target;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use three_d::HeadlessContext;
use tokio::sync::{mpsc, oneshot, Semaphore};
use warp::Filter;
use warp::multipart::FormData;
use warp::reply::Response;

use rs3d::render::RawPixels;

#[derive(Deserialize, Serialize)]
struct Request {
    model: String,
    textures: Vec<String>,
    width: u32,
    height: u32,
}

#[derive(Serialize)]
struct LogEntry {
    level: String,
    target: String,
    message: String,
}

#[tokio::main]
async fn main() {
    init_logger();

    let context = HeadlessContext::new().unwrap();

    let (request_tx, mut request_rx) = mpsc::channel::<(Request, oneshot::Sender<Result<RawPixels>>)>(10);

    tokio::spawn(async move { serve(request_tx).await; });

    loop {
        let (request, response_tx) = request_rx.recv().await.unwrap();
        let pixels = rs3d::render::render(
            request.model,
            request.textures,
            &context,
            request.width,
            request.height,
        ).await;
        let _ = response_tx.send(pixels);
    }
}

async fn serve(request_tx: mpsc::Sender<(Request, oneshot::Sender<Result<RawPixels>>)>) {
    let semaphore = Arc::new(Semaphore::new(1));
    let semaphore_clone = semaphore.clone();
    let request_tx_clone = request_tx.clone();
    let options = warp::multipart::form();
    let options = options.max_length(Some(1024 * 1024 * 1024));
    let render_form = warp::post()
        .and(warp::path("render-form"))
        .and(options)
        .and(warp::body::content_length_limit(u64::MAX))
        .and(warp::any().map(move || semaphore_clone.clone()))
        .and(warp::any().map(move || request_tx_clone.clone()))
        .and_then(|form: FormData, sem: Arc<Semaphore>, request_tx: mpsc::Sender<(Request, oneshot::Sender<Result<RawPixels>>)>|
            async move {
                let fields: Vec<_> = form.and_then(|mut field| async move {
                    let mut bytes: Vec<u8> = Vec::new();
                    while let Some(content) = field.data().await {
                        let content = content.unwrap();
                        bytes.put(content);
                    }
                    Ok((
                        field.name().to_string(),
                        field.filename().unwrap_or("no name given").to_string(),
                        // String::from_utf8_lossy(&*bytes).to_string(),
                    ))
                })
                    .try_collect()
                    .await
                    .unwrap();
                Ok::<_, warp::Rejection>(format!("{:?}", fields))
            });

    let render = warp::post()
        .and(warp::path("render"))
        .and(warp::body::json())
        .and(warp::header::optional("accept"))
        .and(warp::any().map(move || semaphore.clone()))
        .and(warp::any().map(move || request_tx.clone()))
        .and_then(move |r: Request, accept_header: Option<String>, sem: Arc<Semaphore>, request_tx: mpsc::Sender<(Request, oneshot::Sender<Result<RawPixels>>)>| {
            async move {
                let permit = sem.acquire_owned().await.unwrap();

                let (response_tx, response_rx) = oneshot::channel();
                request_tx.try_send((r, response_tx)).unwrap();
                let pixels = response_rx.await.unwrap().unwrap();

                drop(permit);

                if let Some(mime) = accept_header {
                    if mime.contains("image/webp") {
                        let start = std::time::Instant::now();
                        let img = image::load_from_memory(&pixels).unwrap();
                        let mut writer = std::io::Cursor::new(Vec::new());
                        img.write_to(&mut writer, image::ImageOutputFormat::WebP).unwrap();
                        log::info!("Time webp: {:?}", start.elapsed());

                        return Ok::<Response, warp::Rejection>(warp::http::response::Builder::new()
                            .header("Content-Type", "image/webp")
                            .body(writer.into_inner().into())
                            .unwrap());
                    }
                }

                Ok::<Response, warp::Rejection>(warp::http::response::Builder::new()
                    .header("Content-Type", "image/png")
                    .body(pixels.into())
                    .unwrap())
            }
        });

    let health = warp::get()
        .and(warp::path("health"))
        .map(|| "ok");

    let routes = render.or(health).or(render_form);

    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}

fn init_logger() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .target(Target::Stdout)
        .format(|buf, record| {
            let entry = LogEntry {
                level: record.level().to_string(),
                target: record.target().to_string(),
                message: format!("{}", record.args()),
            };
            let content = serde_json::to_string(&entry).unwrap();
            writeln!(buf, "{}", content)
        })
        .init();
}
