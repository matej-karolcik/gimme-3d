use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use anyhow::Result;
use image::DynamicImage;
use three_d::HeadlessContext;
use tokio::sync::{mpsc, oneshot, Semaphore};
use warp::Filter;
use warp::multipart::FormData;
use warp::reply::Response;

use crate::render::*;

use super::{config, logger, request};

type ResultChannel = oneshot::Sender<Result<DynamicImage>>;

pub async fn run() {
    logger::init();

    let config = config::Config::parse_toml("config.toml".to_string()).unwrap_or_default();
    let local_model_dir = config.models.local_model_dir;

    let context = HeadlessContext::new().unwrap();

    let (request_tx, mut request_rx) = mpsc::channel::<(request::Request, ResultChannel)>(10);

    tokio::spawn(async move { serve(config.port, request_tx).await; });

    loop {
        let (request, response_tx) = request_rx.recv().await.unwrap();
        if request.has_raw_textures() {
            let pixels = render_raw_images(
                request.model_url,
                request.model,
                request.textures.unwrap(),
                &context,
                request.width * config.upscale_factor,
                request.height * config.upscale_factor,
                &local_model_dir,
            ).await;
            let _ = response_tx.send(pixels);
            continue;
        }

        let pixels = render_urls(
            request.model_url,
            request.model,
            request.texture_urls.unwrap_or_default(),
            &context,
            request.width * config.upscale_factor,
            request.height * config.upscale_factor,
            &local_model_dir,
        ).await;
        let _ = response_tx.send(pixels);
    }
}

async fn serve(
    port: u16,
    request_tx: mpsc::Sender<(request::Request, ResultChannel)>,
) {
    let semaphore = Arc::new(Semaphore::new(1));
    let semaphore_clone = semaphore.clone();
    let request_tx_clone = request_tx.clone();
    let render_form = warp::post()
        .and(warp::path("render-form"))
        .and(warp::multipart::form().max_length(Some(1024 * 1024 * 1024)))
        .and(warp::header::optional("accept"))
        .and(warp::any().map(move || semaphore_clone.clone()))
        .and(warp::any().map(move || request_tx_clone.clone()))
        .and_then(|form: FormData, accept_header: Option<String>, sem: Arc<Semaphore>, request_tx: mpsc::Sender<(request::Request, ResultChannel)>|
            async move {
                let start = std::time::Instant::now();

                let request_future = request::Request::from_form_data(form).await;
                if let Err(e) = request_future {
                    log::error!("Error: {}", e);
                    return Err(warp::reject::Rejection::from(InternalServerError(e)));
                }

                let r = request_future.unwrap();
                let permit = sem.acquire_owned().await.unwrap();

                let (response_tx, response_rx) = oneshot::channel();

                let width = r.width;
                let height = r.height;

                request_tx.try_send((r, response_tx)).unwrap();
                let pixels = match response_rx.await.unwrap() {
                    Ok(content) => content,
                    Err(e) => {
                        drop(permit);
                        log::error!("Error: {}", e);
                        return Err(warp::reject::Rejection::from(InternalServerError(e)));
                    }
                };

                drop(permit);

                log::info!("Time overall: {:?}", start.elapsed());

                respond(accept_header, pixels, start, width, height)
            });

    let render = warp::post()
        .and(warp::path("render"))
        .and(warp::body::json())
        .and(warp::header::optional("accept"))
        .and(warp::any().map(move || semaphore.clone()))
        .and(warp::any().map(move || request_tx.clone()))
        .and_then(move |r: request::Request, accept_header: Option<String>, sem: Arc<Semaphore>, request_tx: mpsc::Sender<(request::Request, ResultChannel)>| {
            async move {
                let start = std::time::Instant::now();
                let permit = sem.acquire_owned().await.unwrap();

                let (response_tx, response_rx) = oneshot::channel();

                let width = r.width;
                let height = r.height;

                request_tx.try_send((r, response_tx)).unwrap();
                let pixels = response_rx.await.unwrap().unwrap();

                drop(permit);

                respond(accept_header, pixels, start, width, height)
            }
        });

    let health = warp::get()
        .and(warp::path("health"))
        .map(|| "ok");

    let routes = render.or(health).or(render_form);

    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}

fn respond(
    accept_header: Option<String>,
    pixels: DynamicImage,
    start: std::time::Instant,
    width: u32,
    height: u32,
) -> Result<Response, warp::Rejection> {
    let result = if pixels.width() == width && pixels.height() == height {
        pixels
    } else {
        image::imageops::thumbnail(&pixels, width, height).into()
    };

    if let Some(mime) = accept_header {
        if mime.contains("image/webp") {
            let start = std::time::Instant::now();
            // let img = image::load_from_memory(&pixels).unwrap();
            let mut writer = std::io::Cursor::new(Vec::new());
            result.write_to(&mut writer, image::ImageOutputFormat::WebP).unwrap();

            log::info!("Time webp: {:?}", start.elapsed());
            log::info!("Time overall: {:?}", start.elapsed());


            return Ok::<Response, warp::Rejection>(warp::http::response::Builder::new()
                .header("Content-Type", "image/webp")
                .body(writer.into_inner().into())
                .unwrap());
        }
    }

    let mut writer = std::io::Cursor::new(Vec::new());
    result.write_to(&mut writer, image::ImageOutputFormat::Png).unwrap();

    log::info!("Time overall: {:?}", start.elapsed());

    Ok::<Response, warp::Rejection>(warp::http::response::Builder::new()
        .header("Content-Type", "image/png")
        .body(writer.into_inner().into())
        .unwrap())
}

struct InternalServerError(anyhow::Error);

impl Debug for InternalServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InternalServerError")
            .field("error", &self.0)
            .finish()
    }
}

impl warp::reject::Reject for InternalServerError {}
