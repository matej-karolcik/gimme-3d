use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use bytes::BufMut;
use futures_util::TryStreamExt;
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use tokio::sync::{mpsc, oneshot, Semaphore};
use warp::Filter;
use warp::multipart::FormData;
use warp::reply::Response;

use crate::server::request::{ClientError, Request};
use crate::server::server::ResultChannel;

pub fn get() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(warp::path("gimme-3d")).map(|| {
        warp::reply::html(
            r#"<html lang="en">
<head>
    <title>Debug</title>
</head>
<body>
<p>Click on the "Choose File" button to upload files:</p>
<form method="post" enctype="multipart/form-data" target="_blank">
    <label for="mask">Mask</label>
    <input type="file" id="mask" name="mask" accept="image/*" required/><br>
    <label for="model">GLB</label>
    <input type="file" id="model" name="model" required/><br>
    <label for="texture">Texture</label>
    <input type="file" id="texture" name="texture" accept="image/*"/><br>
    <button type="submit">Upload</button>
</form>
</body>
</html>
"#,
        )
    })
}

pub fn post(
    request_tx: mpsc::Sender<(Request, ResultChannel)>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let semaphore = Arc::new(Semaphore::new(1));
    warp::post()
        .and(warp::path("gimme-3d"))
        .and(warp::multipart::form().max_length(Some(1024 * 1024 * 1024)))
        .and(warp::any().map(move || request_tx.clone()))
        .and(warp::any().map(move || semaphore.clone()))
        .and_then(
            |form: FormData,
             request_tx: mpsc::Sender<(Request, ResultChannel)>,
             sem: Arc<Semaphore>| async move {
                let request_future = DebugRequest::from_form_data(form).await;
                let r = request_future.unwrap();

                let (response_tx, response_rx) = oneshot::channel();

                let mask_image = if let Ok(mask) = image::load_from_memory(&r.mask) {
                    mask
                } else {
                    return Err(warp::reject::Rejection::from(InternalServerError(
                        anyhow::anyhow!("Could not load mask"),
                    )));
                };

                let mut request: Request = r.clone().into();
                request.width = mask_image.width();
                request.height = mask_image.height();

                if r.texture.is_none() {
                    let texture_bytes = std::fs::read("testdata/canvas.png").unwrap();
                    request.textures = Some(vec![texture_bytes]);
                }

                let _ = sem.acquire_owned().await.unwrap();
                request_tx.try_send((request, response_tx)).unwrap();
                let pixels = match response_rx.await.unwrap() {
                    Ok(content) => content,
                    Err(e) => {
                        log::error!("Error: {}", e);
                        return Err(warp::reject::Rejection::from(InternalServerError(e)));
                    }
                };

                let result = multiply_vanilla(&mask_image, &pixels);

                respond(result)
            },
        )
}

fn respond(result: DynamicImage) -> anyhow::Result<Response, warp::Rejection> {
    let mut writer = std::io::Cursor::new(Vec::new());
    result
        .write_to(&mut writer, image::ImageOutputFormat::WebP)
        .unwrap();

    Ok::<Response, warp::Rejection>(
        warp::http::response::Builder::new()
            .header("Content-Type", "image/webp")
            .body(writer.into_inner().into())
            .unwrap(),
    )
}

#[derive(Debug, Clone)]
struct DebugRequest {
    model: Vec<u8>,
    mask: Vec<u8>,
    texture: Option<Vec<u8>>,
}

impl Into<Request> for DebugRequest {
    fn into(self) -> Request {
        let textures = match self.texture {
            Some(texture) => Some(vec![texture]),
            None => None,
        };
        Request {
            model: Some(self.model),
            textures,
            ..Default::default()
        }
    }
}

impl DebugRequest {
    pub async fn from_form_data(form: FormData) -> anyhow::Result<Self> {
        let fields: HashMap<String, Vec<u8>> = form
            .and_then(|mut field| async move {
                let mut bytes: Vec<u8> = Vec::new();
                while let Some(content) = field.data().await {
                    let content = content?;
                    bytes.put(content);
                }

                Ok((field.name().to_string(), bytes))
            })
            .try_collect()
            .await?;

        let model = fields
            .get("model")
            .ok_or(ClientError::MissingField("model".to_string()))?
            .to_vec();
        let mask = fields
            .get("mask")
            .ok_or(ClientError::MissingField("mask".to_string()))?
            .to_vec();

        let mut texture: Option<Vec<u8>> = None;
        if let Some(maybe_texture) = fields.get("texture") {
            if !maybe_texture.is_empty() {
                texture = Some(maybe_texture.to_vec());
            }
        }

        Ok(DebugRequest {
            model,
            mask,
            texture,
        })
    }
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

fn multiply_vanilla(bottom: &DynamicImage, top_raw: &DynamicImage) -> DynamicImage {
    let top;
    if top_raw.dimensions() != bottom.dimensions() {
        top = top_raw.resize(
            bottom.width(),
            bottom.height(),
            image::imageops::FilterType::Lanczos3,
        );
    } else {
        top = top_raw.clone();
    }

    let mut result_image = RgbaImage::new(bottom.width(), bottom.height());

    for y in 0..bottom.height() {
        for x in 0..bottom.width() {
            let pixel1 = bottom.get_pixel(x, y);
            let pixel2 = top.get_pixel(x, y);

            let mut result = vec![];
            (0..3).for_each(|i| {
                let ch1 = pixel1[i] as f32 / 255.;
                let ch2 = pixel2[i] as f32 / 255.;

                if ch2 == 0. {
                    result.push((ch1 * 255.) as u8);
                    return;
                }

                result.push((ch1 * ch2 * 255.) as u8);
            });

            result.push(pixel1[3]);

            let product: [u8; 4] = result.try_into().unwrap();
            let product = Rgba(product);

            result_image.put_pixel(x, y, product);
        }
    }

    result_image.into()
}
