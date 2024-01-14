use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
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

use rs3d::error::ServerError;
use rs3d::render::RawPixels;

#[derive(Deserialize, Serialize)]
struct Request {
    model: String,
    texture_urls: Option<Vec<String>>,
    textures: Option<Vec<Vec<u8>>>,
    width: u32,
    height: u32,
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Request")
            .field("model", &self.model)
            .field("textures (length)", &self.textures.is_some())
            .field("texture_urls (length)", &self.texture_urls.is_some())
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl Request {
    fn has_raw_textures(&self) -> bool {
        self.textures.is_some()
    }

    async fn from_form_data(form: FormData) -> Result<Self> {
        let fields: HashMap<String, Vec<u8>> = form.and_then(|mut field| async move {
            let mut bytes: Vec<u8> = Vec::new();
            while let Some(content) = field.data().await {
                let content = content?;
                bytes.put(content);
            }

            Ok((field.name().to_string(), bytes))
        })
            .try_collect()
            .await?;

        let model = String::from_utf8(fields.get("model")
            .ok_or(ServerError::MissingField("model".to_string()))?.to_vec())?;
        let width = String::from_utf8(fields.get("width")
            .ok_or(ServerError::MissingField("width".to_string()))?.to_vec())?
            .parse()?;
        let height = String::from_utf8(fields.get("height")
            .ok_or(ServerError::MissingField("height".to_string()))?.to_vec())?
            .parse()?;

        let mut textures = Vec::new();
        fields.iter()
            .filter(|(k, _)| k.starts_with("texture"))
            .for_each(|(_, v)| textures.push(v.to_vec()));

        Ok(Request {
            model,
            texture_urls: None,
            textures: Some(textures),
            width,
            height,
        })
    }
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

    // todo replace this with .env
    std::env::set_var("LOCAL_MODEL_DIR", "glb");
    let local_model_dir = get_local_model_dir();

    let context = HeadlessContext::new().unwrap();

    let (request_tx, mut request_rx) = mpsc::channel::<(Request, oneshot::Sender<Result<RawPixels>>)>(10);

    tokio::spawn(async move { serve(request_tx).await; });

    loop {
        let (request, response_tx) = request_rx.recv().await.unwrap();
        if request.has_raw_textures() {
            let pixels = rs3d::render::render_raw_images(
                request.model,
                request.textures.unwrap(),
                &context,
                request.width,
                request.height,
                &local_model_dir,
            ).await;
            let _ = response_tx.send(pixels);
            continue;
        }
        let pixels = rs3d::render::render_urls(
            request.model,
            request.texture_urls.unwrap_or_default(),
            &context,
            request.width,
            request.height,
            &local_model_dir,
        ).await;
        let _ = response_tx.send(pixels);
    }
}


async fn serve(request_tx: mpsc::Sender<(Request, oneshot::Sender<Result<RawPixels>>)>) {
    let semaphore = Arc::new(Semaphore::new(1));
    let semaphore_clone = semaphore.clone();
    let request_tx_clone = request_tx.clone();
    let render_form = warp::post()
        .and(warp::path("render-form"))
        .and(warp::multipart::form().max_length(Some(1024 * 1024 * 1024)))
        .and(warp::header::optional("accept"))
        .and(warp::any().map(move || semaphore_clone.clone()))
        .and(warp::any().map(move || request_tx_clone.clone()))
        .and_then(|form: FormData, accept_header: Option<String>, sem: Arc<Semaphore>, request_tx: mpsc::Sender<(Request, oneshot::Sender<Result<RawPixels>>)>|
            async move {
                let start = std::time::Instant::now();

                let r = Request::from_form_data(form).await.unwrap();
                let permit = sem.acquire_owned().await.unwrap();

                let (response_tx, response_rx) = oneshot::channel();
                request_tx.try_send((r, response_tx)).unwrap();
                let pixels = response_rx.await.unwrap().unwrap();

                drop(permit);

                log::info!("Time overall: {:?}", start.elapsed());

                respond(accept_header, pixels)
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

                respond(accept_header, pixels)
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
        .filter_level(log::LevelFilter::Info)
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

fn get_local_model_dir() -> String {
    std::env::var("LOCAL_MODEL_DIR")
        .unwrap_or_else(|_| String::new())
}

fn respond(accept_header: Option<String>, pixels: RawPixels) -> Result<Response, warp::Rejection> {
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
