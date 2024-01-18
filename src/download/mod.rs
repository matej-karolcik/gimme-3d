use std::path::{Path, PathBuf};

use anyhow::Result;
use async_trait::async_trait;
use clap::{Arg, ArgMatches, Command};
use futures_util::{stream, StreamExt};
use indicatif::ProgressBar;
use url::Url;

use crate::server;

pub struct Download {}

#[async_trait]
impl crate::Subcommand for Download {
    fn get_subcommand(&self) -> Command {
        Command::new("download")
            .arg(
                Arg::new("config")
                    .required(true)
            )
    }

    async fn run(&self, matches: &ArgMatches) -> Result<()> {
        let config_path = matches.get_one::<String>("config").unwrap();
        let config = server::config::Config::parse_toml(config_path.to_string()).unwrap();
        models(
            config.models.models_base_url,
            config.models.models,
            config.models.local_model_dir,
        ).await
    }
}

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

    let pb = ProgressBar::new(models.len() as u64);

    stream::iter(models)
        .for_each_concurrent(10, |model| {
            let base = base.clone();
            let url = base.join(model.as_str()).unwrap();
            let output = output.join(model.as_str()).to_owned();
            async {
                download(url, output).await.unwrap();
                pb.clone().inc(1);
            }
        })
        .await;

    pb.finish_with_message("done");

    Ok(())
}

async fn download(url: Url, output: PathBuf) -> Result<()> {
    let start = std::time::Instant::now();
    let response = reqwest::get(url).await?;
    let mut file = std::fs::File::create(output.clone())?;

    let content = response.text().await?;
    std::io::copy(&mut content.as_bytes(), &mut file)?;

    println!("downloaded {} in {:?}", output.to_str().unwrap(), start.elapsed());

    Ok(())
}
