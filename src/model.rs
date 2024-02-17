use std::path::Path;

use anyhow::{anyhow, Result};
use three_d_asset::io::RawAssets;

use crate::error::Error;

pub(crate) async fn load(model_path: &String, local_model_path: &String) -> Result<(RawAssets, String)> {
    let mut final_model_path = model_path.clone();
    let mut loaded_assets;

    if let Ok(model_path) = get_local_model(local_model_path, &model_path.clone()) {
        loaded_assets = three_d_asset::io::load(&[model_path.clone()]).unwrap();
        final_model_path = model_path.clone();
        log::info!("loaded local model: {}", model_path);
    } else {
        // todo fix this somehow
        // let model_bytes = download(model_path.clone()).await?;

        // loaded_assets = RawAssets::new();
        // loaded_assets.insert(final_model_path.clone(), model_bytes);
        log::info!("loaded remote model: {}", model_path);
        let to_load = vec![model_path.clone()];
        let loaded_future = three_d_asset::io::load_async(to_load.as_slice());

        loaded_assets = loaded_future.await.unwrap();
    }

    Ok((loaded_assets, final_model_path))
}

pub async fn download(url: String) -> Result<Vec<u8>> {
    if !url.starts_with("http") {
        return Err(anyhow!("url does not start with http"));
    }

    let url = reqwest::Url::parse(url.as_str())?;
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(Error::ModelDownloadError {
            status_code: response.status(),
            message: response.text().await.unwrap_or("".to_owned()),
        }.into());
    }

    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}


fn get_local_model(local_dir: &String, path: &String) -> Result<String> {
    if local_dir.is_empty() || path.is_empty() {
        return Err(anyhow!("local directory or path is empty"));
    }

    let local_dir = Path::new(local_dir.as_str());
    let model_path = Path::new(path.as_str());
    let filename = model_path.file_name();

    if let None = filename {
        return Err(anyhow!("no filename found in {}", path));
    }

    let model_path = local_dir.join(filename.unwrap());

    if !model_path.exists() {
        log::info!("no local model found: {}", model_path.display());
        return Err(Error::NoLocalModel(model_path.display().to_string()).into());
    }


    Ok(String::from(model_path.to_str().unwrap()))
}
