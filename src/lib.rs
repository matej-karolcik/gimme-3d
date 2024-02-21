use async_trait::async_trait;
use clap::Command;

pub mod collect;
pub mod download;
pub mod error;
pub mod fbx2gltf;
pub mod gltf;
pub mod img;
pub mod model;
pub mod object;
pub mod render;
pub mod render_file;
pub mod server;

#[async_trait]
pub trait Subcommand {
    fn get_subcommand(&self) -> Command;
    async fn run(&self, matches: &clap::ArgMatches) -> anyhow::Result<()>;
}
