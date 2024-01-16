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
    pub model: String,
    pub texture_urls: Option<Vec<String>>,
    pub textures: Option<Vec<Vec<u8>>>,
    pub width: u32,
    pub height: u32,
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
    pub fn has_raw_textures(&self) -> bool {
        self.textures.is_some()
    }

    pub async fn from_form_data(form: FormData) -> anyhow::Result<Self> {
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
            .ok_or(ClientError::MissingField("model".to_string()))?.to_vec())?;
        let width = String::from_utf8(fields.get("width")
            .ok_or(ClientError::MissingField("width".to_string()))?.to_vec())?
            .parse()?;
        let height = String::from_utf8(fields.get("height")
            .ok_or(ClientError::MissingField("height".to_string()))?.to_vec())?
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

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Error while parsing form data: {0}")]
    MissingField(String),
}
