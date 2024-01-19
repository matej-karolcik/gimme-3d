use clap::{Arg, Command};

use rs3d::{collect, download, fbx2gltf, render_file, server, Subcommand};

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
                )
                .arg(
                    Arg::new("results")
                        .default_value("results")
                )
                .about("Render a single glb/gltf file or directory containing multiple")
        );

    let mut debug_components: Vec<Box<dyn Subcommand>> = vec![];

    if Ok("release".to_owned()) != std::env::var("PROFILE") {
        debug_components.push(Box::new(collect::Collect {}));
        debug_components.push(Box::new(download::Download {}));
        debug_components.push(Box::new(fbx2gltf::Fbx2Gltf {}));

        for component in &debug_components {
            root = root.subcommand(component.get_subcommand());
        }
    }

    match root.get_matches().subcommand() {
        Some(("serve", _)) => {
            server::run().await;
        }
        Some(("render", submatches)) => {
            let context = three_d::HeadlessContext::new().unwrap();
            let input = submatches.get_one::<String>("input").unwrap();
            let results = submatches.get_one::<String>("results").unwrap();
            render_file::run_multiple(input, results, &context).await;
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
