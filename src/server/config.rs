use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub port: u16,
    pub models: Models,
}

#[derive(Deserialize)]
pub struct Models {
    pub local_model_dir: String,
    pub models_base_url: String,
    pub models: Vec<String>,
}

impl Config {
    pub fn parse_toml(path: String) -> Result<Self> {
        let config_file = std::fs::read_to_string(path)?;
        Ok(Self::parse(config_file)?)
    }

    fn parse(config: String) -> Result<Self> {
        let config: Config = toml::from_str(config.as_str())?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn test_parse_toml() -> Result<()> {
        let config = Config::parse_toml("config.test.toml".to_string())?;

        assert_eq!(config.port, 3030);
        assert_eq!(config.models.local_model_dir, "/var/models/");
        assert_eq!(config.models.models_base_url, "https://jq-staging-matko.s3.eu-central-1.amazonaws.com/gltf/");
        assert_eq!(config.models.models.len(), 2);
        assert_eq!(config.models.models[0], "foo.glb");
        assert_eq!(config.models.models[1], "bar.glb");

        Ok(())
    }
}
