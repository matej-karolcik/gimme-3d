fn main() {
    let bytes = std::fs::read("test2.png").unwrap();
    let img = image::load_from_memory(&bytes).unwrap();
    // let mut file = std::fs::File::create("test2.webp").unwrap();

    let mut writer = std::io::Cursor::new(Vec::new());

    img.write_to(&mut writer, image::ImageOutputFormat::WebP).unwrap();
}
