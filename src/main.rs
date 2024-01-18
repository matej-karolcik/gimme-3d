use clap::{Arg, Command};

use rs3d::{download, fbx2gltf, server, Subcommand};

#[tokio::main]
async fn main() {
    let mut root = Command::new("preview");

    let mut components: Vec<Box<dyn Subcommand>> = vec![

    ];
    components.push(Box::new(download::Download {}));
    components.push(Box::new(fbx2gltf::Fbx2Gltf {}));

    for component in components {
        let subcmd = component.get_subcommand();
        root = root.subcommand(component.get_subcommand());
    }

    let matches = Command::new("preview")
        .subcommand(
            Command::new("collect-models")
                .arg(
                    Arg::new("input-dir")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("serve")
                .arg(
                    Arg::new("port")
                        .value_parser(clap::value_parser!(u16).range(3000..))
                )
        )
        .subcommand(
            Command::new("download")
                .arg(
                    Arg::new("config")
                        .required(true)
                )
        )
        .get_matches();

    match matches.subcommand() {
        Some(("collect-models", submatches)) => {
            env_logger::Builder::new()
                .filter_level(log::LevelFilter::Info)
                .target(env_logger::Target::Stdout)
                .init();

            let input_dir = submatches.get_one::<String>("input-dir").unwrap();
            collect_models(input_dir);
        }
        Some(("serve", submatches)) => {
            let port = submatches.get_one::<u16>("port").unwrap_or_else(|| &3030);
            server::run(*port).await;
        }
        Some(("download", submatches)) => {
            let config_path = submatches.get_one::<String>("config").unwrap();
            let config = server::config::Config::parse_toml(config_path.to_string()).unwrap();
            download::models(
                config.models.models_base_url,
                config.models.models,
                config.models.local_model_dir,
            ).await.unwrap();
        }
        Some((&_, _)) => {}
        None => {}
    }
}


fn collect_models(input_dir: &String) {
    let models = std::fs::read_dir(input_dir)
        .unwrap()
        .filter_map(|file| {
            let file = match file {
                Ok(file) => file,
                Err(_) => { return None; }
            };

            let path = file.path();

            if path.extension().is_none() {
                return None;
            }

            let extension = path.extension()?;
            if extension != "glb" {
                return None;
            }

            let path = path.file_name()?;

            Some(path.to_str().unwrap().to_string())
        })
        .collect::<Vec<String>>();

    let result_path = "models.txt";
    std::fs::write(result_path, models.join("\n")).unwrap();

    log::info!("wrote models to {}", result_path);
}
