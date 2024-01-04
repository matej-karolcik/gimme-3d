use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Asset loading error: {0}")]
    AssetLoadingError(three_d_asset::Error),

    #[error("Gltf parsing error: {0}")]
    GltfParsingError(gltf::Error),

    #[error("No default scene")]
    NoDefaultScene,

    #[error("No camera")]
    NoCamera,

    #[error("No mesh")]
    NoMesh,
}
