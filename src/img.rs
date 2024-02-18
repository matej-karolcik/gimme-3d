use image::DynamicImage;
use three_d_asset::{Texture2D, TextureData};

use crate::error::Error;

pub fn decode_img(bytes: &[u8]) -> anyhow::Result<Texture2D> {
    let img = image::load_from_memory(bytes)?;

    let width = img.width();
    let height = img.height();

    let data = match img {
        DynamicImage::ImageLuma8(_) => TextureData::RU8(img.into_bytes()),
        DynamicImage::ImageLumaA8(img) => TextureData::RgU8(
            img.into_raw()
                .chunks(2)
                .map(|c| [c[0], c[1]])
                .collect::<Vec<_>>(),
        ),
        DynamicImage::ImageRgb8(img) => TextureData::RgbU8(
            img.into_raw()
                .chunks(3)
                .map(|c| [c[0], c[1], c[2]])
                .collect::<Vec<_>>(),
        ),
        DynamicImage::ImageRgba8(img) => TextureData::RgbaU8(
            img.into_raw()
                .chunks(4)
                .map(|c| [c[0], c[1], c[2], c[3]])
                .collect::<Vec<_>>(),
        ),
        _ => unimplemented!(),
    };

    Ok(Texture2D {
        data,
        width,
        height,
        ..Default::default()
    })
}

pub async fn download_img(url: String) -> anyhow::Result<Texture2D> {
    if !url.starts_with("http") {
        return decode_img(std::fs::read(url)?.as_slice());
    }

    let response = reqwest::get(url).await?;
    if !response.status().is_success() {
        return Err(Error::ImageDownloadError {
            status_code: response.status(),
            message: response.text().await.unwrap_or("".to_owned()),
        }.into());
    }

    let bytes = response.bytes().await?;
    decode_img(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_img() {
        for ext in ["png", "jpg", "webp"] {
            let image_path = format!("testdata/test.{}", ext);
            let bytes = std::fs::read(&image_path).unwrap();
            let _ = decode_img(&bytes).unwrap();
        }
    }
}
