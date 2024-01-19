use anyhow::Result;
use async_trait::async_trait;
use clap::{Arg, ArgMatches, Command};

pub struct Collect {}

#[async_trait]
impl crate::Subcommand for Collect {
    fn get_subcommand(&self) -> Command {
        Command::new("collect")
            .arg(
                Arg::new("input-dir")
                    .required(true)
            )
            .about("Collect model names from a local directory and save them in models.txt (for a later use in config.toml)")
    }

    async fn run(&self, matches: &ArgMatches) -> Result<()> {
        let input_dir = matches.get_one::<String>("input-dir").unwrap();

        async {
            Ok(collect_models(input_dir))
        }.await
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

    println!("wrote models to {}", result_path);
}
