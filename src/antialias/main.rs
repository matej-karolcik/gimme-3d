use std::io::BufWriter;
use std::num::NonZeroU32;

use fast_image_resize as fr;
use image::{ColorType, ImageEncoder};
use image::codecs::webp::WebPEncoder;

fn main() {
    let img = image::open("results/foobar.webp").unwrap();
    let start = std::time::Instant::now();
    let width = NonZeroU32::new(img.width()).unwrap();
    let height = NonZeroU32::new(img.height()).unwrap();
    let mut src_image = fr::Image::from_vec_u8(
        width,
        height,
        img.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    ).unwrap();

    let alpha_mul_div = fr::MulDiv::default();
    alpha_mul_div
        .multiply_alpha_inplace(&mut src_image.view_mut())
        .unwrap();

    let dst_width = NonZeroU32::new(2000).unwrap();
    let dst_height = NonZeroU32::new(2000).unwrap();
    let mut dst_image = fr::Image::new(
        dst_width,
        dst_height,
        src_image.pixel_type(),
    );

    let mut dst_view = dst_image.view_mut();
    let mut resizer = fr::Resizer::new(
        fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3),
    );

    resizer.resize(&src_image.view(), &mut dst_view).unwrap();
    alpha_mul_div.divide_alpha_inplace(&mut dst_view).unwrap();

    let mut result_buf = BufWriter::new(Vec::new());
    WebPEncoder::new(&mut result_buf)
        .write_image(
            dst_image.buffer(),
            dst_width.get(),
            dst_height.get(),
            ColorType::Rgba8,
        )
        .unwrap();

    println!("Time resize: {:?}", std::time::Instant::now() - start);
    std::fs::write("results/aa.webp", result_buf.into_inner().unwrap()).unwrap();
}

fn resize() {

}
