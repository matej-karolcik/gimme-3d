use std::io::Write;
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
                    .long_help("path to config.toml to be used")
            )
            .about("Download models from a remote server to a local directory (for caching)")
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
    mut base_url: String,
    models: Vec<String>,
    output_dir: String,
) -> Result<()> {
    if !base_url.ends_with("/") {
        base_url = format!("{}/", base_url);
    }
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
                download(url, output, &pb).await.unwrap();
                pb.clone().inc(1);
            }
        })
        .await;

    pb.finish_with_message("done");

    Ok(())
}

async fn download(url: Url, output: PathBuf, pb: &ProgressBar) -> Result<()> {
    let start = std::time::Instant::now();
    let mut response = reqwest::get(url).await?;
    let mut file = std::fs::File::create(output.clone())?;

    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk)?;
    }

    pb.set_message(format!(
        "Downloaded {} in {}ms",
        output.to_str().unwrap(),
        start.elapsed().as_millis()
    ));

    Ok(())
}
