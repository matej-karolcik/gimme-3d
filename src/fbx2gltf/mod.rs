use std::path::Path;
use std::process::ExitStatus;

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use clap::ArgAction::SetTrue;
use clap::{Arg, ArgMatches, Command};

pub struct Fbx2Gltf {}

#[async_trait]
impl crate::Subcommand for Fbx2Gltf {
    fn get_subcommand(&self) -> Command {
        Command::new("convert")
            .arg(
                Arg::new("input")
                    .short('i')
                    .long("input")
                    .required(true)
                    .help("input file or directory"),
            )
            .arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .required(false)
                    .default_value("output")
                    .help("output directory"),
            )
            .arg(
                Arg::new("binary")
                    .short('b')
                    .long("binary")
                    .required(false)
                    .action(SetTrue)
                    .help("output binary gltf"),
            )
            .about("Convert fbx files into glb/gltf")
    }

    async fn run(&self, matches: &ArgMatches) -> anyhow::Result<()> {
        let input = matches.get_one::<String>("input").unwrap();
        let output = matches.get_one::<String>("output").unwrap();
        let binary = matches.get_flag("binary");

        convert(input, output, binary)?;

        std::future::ready(Ok(())).await
    }
}

pub fn convert(input: &String, output: &String, binary: bool) -> anyhow::Result<()> {
    let input_path = Path::new(input);
    if !input_path.exists() {
        return Err(anyhow!("input {} does not exist", input));
    }

    let output_path = Path::new(output);
    if !output_path.exists() {
        std::fs::create_dir_all(output_path)
            .context(format!("failed to create directory {}", output))?;
    }

    if input_path.is_dir() {
        input_path
            .read_dir()
            .context("cannot list input directory")?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() {
                    Some(path)
                } else {
                    None
                }
            })
            .for_each(|path| {
                convert_file(&path, Some(output_path), binary)
                    .map_err(|e| anyhow!("failed to convert file {}: {}", path.display(), e))
                    .unwrap();
            });
    } else {
        convert_file(input_path, Some(output_path), binary)?;
    }

    Ok(())
}

fn convert_file(
    input_path: &Path,
    output_path: Option<&Path>,
    binary_output: bool,
) -> anyhow::Result<ExitStatus> {
    // todo use std::process::Command::new("fbx2gltf-bin") when it is available
    // todo prolly using which crate
    let binding = std::process::Command::new("./fbx2gltf-bin");
    let mut cmd = binding;
    cmd.arg(input_path.to_str().unwrap());

    if binary_output {
        cmd.arg("-b");
    }

    if let Some(output_path) = output_path {
        let result_path = output_path.join(input_path.file_stem().unwrap());
        cmd.args(["-o", result_path.to_str().unwrap()]);
    }

    println!("running: {:?}", cmd);
    cmd.spawn()?.wait().map_err(anyhow::Error::new)
}
