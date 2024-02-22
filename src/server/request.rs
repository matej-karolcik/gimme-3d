use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

use bytes::BufMut;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use warp::multipart::FormData;

#[derive(Deserialize, Serialize)]
pub struct Request {
    pub model_url: Option<String>,
    pub model: Option<Vec<u8>>,
    // todo these should just be vecs
    pub texture_urls: Option<Vec<String>>,
    pub textures: Option<Vec<Vec<u8>>>,
    pub width: u32,
    pub height: u32,
}

impl Default for Request {
    fn default() -> Self {
        Request {
            model_url: None,
            model: None,
            texture_urls: None,
            textures: None,
            width: 0,
            height: 0,
        }
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Request")
            .field("model", &self.model.is_some())
            .field("model_url", &self.model_url)
            .field("textures (length)", &self.textures.is_some())
            .field("texture_urls (length)", &self.texture_urls.is_some())
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl Request {
    pub fn has_raw_textures(&self) -> bool {
        self.textures.is_some()
    }

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

        let mut model_url: Option<String> = None;
        let maybe_model_url = fields.get("model_url");
        if let Some(maybe_model_url) = maybe_model_url {
            if !maybe_model_url.is_empty() {
                model_url = Some(String::from_utf8(maybe_model_url.to_vec())?);
            }
        }
        let mut model: Option<Vec<u8>> = None;
        if let Some(maybe_model) = fields.get("model") {
            if !maybe_model.is_empty() {
                model = Some(maybe_model.to_vec());
            }
        }

        if model_url.is_none() && model.is_none() {
            return Err(anyhow::anyhow!("model_url or model field is required"));
        }

        let width = String::from_utf8(
            fields
                .get("width")
                .ok_or(ClientError::MissingField("width".to_string()))?
                .to_vec(),
        )?
        .parse()?;
        let height = String::from_utf8(
            fields
                .get("height")
                .ok_or(ClientError::MissingField("height".to_string()))?
                .to_vec(),
        )?
        .parse()?;

        let mut textures = Vec::new();
        fields
            .iter()
            .filter(|(k, _)| k.starts_with("texture"))
            .for_each(|(_, v)| textures.push(v.to_vec()));

        Ok(Request {
            model,
            model_url,
            texture_urls: None,
            textures: Some(textures),
            width,
            height,
        })
    }
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Error while parsing form data: {0}")]
    MissingField(String),
}
