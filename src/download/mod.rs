use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use futures_util::{stream, StreamExt};
use url::Url;

pub async fn models(
    base_url: String,
    models: Vec<String>,
    output_dir: String,
) -> Result<()> {
    let base = Url::parse(base_url.as_str())?;
    let output = Path::new(output_dir.as_str());

    if !output.exists() {
        std::fs::create_dir_all(output)?;
    }

    stream::iter(models)
        .for_each_concurrent(10, |model| {
            let base = base.clone();
            let url = base.join(model.as_str()).unwrap();
            let output = output.join(model.as_str()).to_owned();
            async move {
                download(url, output).await.unwrap();
            }
        })
        .await;

    Ok(())
}

async fn download(url: Url, output: PathBuf) -> Result<()> {
    let mut response = reqwest::get(url).await?;
    let mut file = std::fs::File::create(output)?;
    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk)?;
    }

    Ok(())
}
