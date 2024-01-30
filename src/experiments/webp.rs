use std::io::Cursor;

use image::DynamicImage;
use image::io::Reader;
use three_d_asset::{Texture2D, TextureData};

use gimme_3d::error::Error;

#[tokio::main]
async fn main() {
    let start = std::time::Instant::now();
    let urls = vec![
        String::from("https://images.unsplash.com/photo-1704372569833-c9f60f52eda3?q=80&w=3387&auto=format&fit=crop&ixlib=rb-4.0.3&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D"),
        String::from("https://images.unsplash.com/photo-1704026438453-fde2ceb923ad?q=80&w=3436&auto=format&fit=crop&ixlib=rb-4.0.3&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D"),
        String::from("https://images.unsplash.com/photo-1683009427037-c5afc2b8134d?q=80&w=3540&auto=format&fit=crop&ixlib=rb-4.0.3&ixid=M3wxMjA3fDF8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D"),
    ];

    let futures = urls.iter().map(|url| tokio::spawn(download_img(url.clone())));

    let results: Vec<_> = futures_util::future::join_all(futures)
        .await
        .into_iter()
        .map(|result| result.unwrap())
        .collect();

    results.iter().for_each(|t| {
        println!("{:?}", t);
    });

    println!("parallel: {:?}", start.elapsed());
    let start = std::time::Instant::now();

    for url in urls.iter() {
        let result = download_img(url.clone()).await;
        println!("{:?}", result);
    }

    println!("sequential: {:?}", start.elapsed());
}

pub async fn download_img(url: String) -> anyhow::Result<Texture2D> {
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

pub fn decode_img(bytes: &[u8]) -> anyhow::Result<Texture2D> {
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
