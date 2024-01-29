use std::io::Cursor;

use image::DynamicImage;
use image::io::Reader;
use three_d_asset::{Texture2D, TextureData};

fn main() {
    for ext in ["png", "jpg", "webp"] {
        let image_path = format!("testdata/test.{}", ext);
        let bytes = std::fs::read(&image_path).unwrap();
        let _ = decode_img(&bytes).unwrap();
    }
}

fn decode_img(bytes: &[u8]) -> anyhow::Result<Texture2D> {
    let reader = Reader::new(Cursor::new(bytes))
        .with_guessed_format()
        .expect("Cursor io never fails");
    let img: DynamicImage = reader.decode()?;

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
