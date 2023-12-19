use std::path::Path;
use std::process::ExitStatus;

use anyhow::{anyhow, Context, Result};
use clap::{Arg, Command};
use clap::ArgAction::SetTrue;

fn main() -> Result<()> {
    let m = Command::new("fbx2gltf")
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
        );

    let matches = m.get_matches();
    let input = matches.get_one::<String>("input").unwrap();
    let output = matches.get_one::<String>("output").unwrap();
    let binary = matches.get_flag("binary");

    let input_path = Path::new(input);
    if !input_path.exists() {
        return Err(anyhow!("input {} does not exist", input));
    }

    let output_path = Path::new(output);
    if !output_path.exists() {
        std::fs::create_dir_all(output_path).with_context(|| format!("failed to create directory {}", output))?;
    }

    if input_path.is_dir() {
        input_path.read_dir().context("cannot list input directory")?
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
                convert_file(&path, Some(&output_path), binary)
                    .map_err(|e| anyhow!("failed to convert file {}: {}", path.display(), e))
                    .unwrap();
            });
    } else {
        convert_file(input_path, Some(&output_path), binary)?;
    }

    Ok(())
}

fn convert_file(input_path: &Path, output_path: Option<&Path>, binary_output: bool) -> Result<ExitStatus> {
    let binding = std::process::Command::new("./fbx2gltf-bin");
    let mut cmd = binding;
    cmd.arg(input_path.to_str().unwrap());

    if binary_output {
        cmd.arg("-b");
    }

    if let Some(output_path) = output_path {
        let result_path = output_path.join(input_path.file_stem().unwrap());
        cmd.args(&["-o", result_path.to_str().unwrap()]);
    }

    println!("running: {:?}", cmd);
    cmd.spawn()?.wait().map_err(anyhow::Error::new)
}
