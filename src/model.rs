use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use three_d_asset::io::RawAssets;

use crate::error::Error;

pub async fn load(
    model_path: Option<String>,
    local_model_dir: &String,
    model_bytes: Option<Vec<u8>>,
) -> Result<(RawAssets, String)> {
    let final_model_path;
    let loaded_assets;

    if model_path.is_none() && model_bytes.is_none() {
        return Err(anyhow!("model path and model bytes are empty"));
    }

    if let Some(model_bytes) = model_bytes {
        final_model_path = if model_path.is_none() {
            "tmp_model.glb".to_string()
        } else {
            let model_path = model_path.unwrap();
            Path::new(local_model_dir)
                .join(
                    Path::new(model_path.as_str())
                        .file_name()
                        .ok_or(anyhow!("no filename found in {}", model_path))?,
                )
                .to_str()
                .ok_or(anyhow!("model path is not valid utf-8"))?
                .to_string()
        };

        // todo
        std::fs::write(final_model_path.clone(), model_bytes.clone())?;
        loaded_assets = three_d_asset::io::load(&[final_model_path.clone()])?;

        return Ok((loaded_assets, final_model_path));
    }

    let model_path = model_path.unwrap();

    if let Ok(model_path) = get_local_model(local_model_dir, &model_path.clone()) {
        loaded_assets = three_d_asset::io::load(&[model_path.clone()])?;
        final_model_path = model_path.clone();
    } else {
        let model_bytes = download(model_path.clone()).await?;
        let model_path = Path::new(local_model_dir).join(
            Path::new(model_path.as_str())
                .file_name()
                .ok_or(anyhow!("no filename found in {}", model_path))?,
        );

        std::fs::write(model_path.clone(), model_bytes.clone())?;
        loaded_assets = three_d_asset::io::load(&[model_path.clone()])?;
        final_model_path = model_path
            .to_str()
            .ok_or(anyhow!("model path is not valid utf-8"))?
            .to_string();
    }

    Ok((loaded_assets, final_model_path))
}

pub async fn download(url: String) -> Result<Vec<u8>> {
    if !url.starts_with("http") {
        return std::fs::read(url).map_err(|e| e.into());
    }

    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(Error::ModelDownloadError {
            status_code: response.status(),
            message: response.text().await.unwrap_or("".to_owned()),
        }
        .into());
    }

    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}

fn get_local_model(local_dir: &String, path: &String) -> Result<String> {
    if path.is_empty() {
        return Err(anyhow!("model path is empty"));
    }

    let model_path = PathBuf::new().join(local_dir).join(path);

    if !model_path.exists() {
        return Err(Error::NoLocalModel(model_path.display().to_string()).into());
    }

    Ok(String::from(
        model_path
            .to_str()
            .ok_or(anyhow!("model path is not valid utf-8"))?,
    ))
}
