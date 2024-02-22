use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Asset loading error: {0}")]
    AssetLoadingError(three_d_asset::Error),

    #[error("Model loading error: {0}")]
    ModelLoadingError(std::io::Error),

    #[error("Model not found: {0}")]
    ModelNotFound(three_d_asset::Error),

    #[error("Gltf parsing error: {0}")]
    GltfParsingError(gltf::Error),

    #[error("No default scene")]
    NoDefaultScene,

    #[error("No camera")]
    NoCamera,

    #[error("No mesh")]
    NoMesh,

    #[error("No local model found at: {0}")]
    NoLocalModel(String),

    #[error("Could not download image: {status_code} -> {message}")]
    ImageDownloadError {
        status_code: reqwest::StatusCode,
        message: String,
    },

    #[error("Could not download model: {status_code} -> {message}")]
    ModelDownloadError {
        status_code: reqwest::StatusCode,
        message: String,
    },

    #[error("No textures")]
    NoTextures,
}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Error while parsing form data: {0}")]
    MissingField(String),
}
