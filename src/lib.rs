use async_trait::async_trait;
use clap::Command;

pub mod object;
pub mod gltf;
pub mod render_file;
pub mod error;
pub mod server;
pub mod render;
pub mod fbx2gltf;
pub mod download;
pub mod collect;
pub mod img;


#[async_trait]
pub trait Subcommand {
    fn get_subcommand(&self) -> Command;
    async fn run(&self, matches: &clap::ArgMatches) -> anyhow::Result<()>;
}
