use std::path::Path;

use clap::{Arg, Command};

use gimme_3d::{collect, download, fbx2gltf, render_file, server, Subcommand};

#[tokio::main]
async fn main() {
    let mut root = Command::new("preview")
        .subcommand(
            Command::new("serve")
                .about("Start http server (using config.toml for configuration)")
        )
        .subcommand(
            Command::new("render")
                .arg(
                    Arg::new("input")
                        .default_value("glb")
                        .long_help("input file or directory")
                )
                .arg(
                    Arg::new("results")
                        .default_value("results")
                        .long_help("output directory, will be created if not present")
                )
                .arg(
                    Arg::new("texture_url")
                        .long_help("texture url to be used, local or remote")
                )
                .about("Render a single glb/gltf file or directory containing multiple")
        );

    let mut debug_components: Vec<Box<dyn Subcommand>> = vec![];

    debug_components.push(Box::new(download::Download {}));
    debug_components.push(Box::new(collect::Collect {}));
    debug_components.push(Box::new(fbx2gltf::Fbx2Gltf {}));

    for component in &debug_components {
        root = root.subcommand(component.get_subcommand());
    }

    match root.get_matches().subcommand() {
        Some(("serve", _)) => {
            server::run().await;
        }
        Some(("render", submatches)) => {
            let context = three_d::HeadlessContext::new().unwrap();
            let input = submatches.get_one::<String>("input").unwrap();
            let results = submatches.get_one::<String>("results").unwrap();
            let texture_url = submatches.get_one::<String>("texture_url");

            let input_path = Path::new(input);

            if input_path.is_dir() {
                render_file::run_multiple(input, results, &context, &texture_url).await;
            } else {
                render_file::run(input, results, &context, &texture_url).await;
            }
        }
        Some((subcommand, submatches)) => {
            for component in &debug_components {
                if component.get_subcommand().get_name() == subcommand {
                    component.run(submatches).await.unwrap();
                }
            }
        }
        None => {}
    }
}
