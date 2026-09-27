use raw_pipeline::frame::{OutputFormat, RenderOptions};

pub fn rgb8_opts(max_edge: u32) -> RenderOptions {
    RenderOptions {
        max_edge,
        output: OutputFormat::Rgb8,
        ..Default::default()
    }
}

pub fn decode_jpeg_rgb(jpeg: &[u8]) -> (Vec<u8>, usize, usize) {
    let img: turbojpeg::Image<Vec<u8>> =
        turbojpeg::decompress(jpeg, turbojpeg::PixelFormat::RGB).unwrap();
    (img.pixels, img.width, img.height)
}
