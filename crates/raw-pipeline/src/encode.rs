use crate::PipelineError;

pub fn encode_jpeg(rgb: &[u8], width: u32, height: u32, quality: i32) -> crate::PipelineResult<Vec<u8>> {
    let image = turbojpeg::Image {
        pixels: rgb,
        width: width as usize,
        pitch: width as usize * 3,
        height: height as usize,
        format: turbojpeg::PixelFormat::RGB,
    };

    turbojpeg::compress(image, quality, turbojpeg::Subsamp::Sub2x2)
        .map(|buf| buf.to_vec())
        .map_err(|e| PipelineError::Encode(format!("{e}")))
}

pub fn encode_jpeg_rgba(rgba: &[u8], width: u32, height: u32, quality: i32) -> crate::PipelineResult<Vec<u8>> {
    let image = turbojpeg::Image {
        pixels: rgba,
        width: width as usize,
        pitch: width as usize * 4,
        height: height as usize,
        format: turbojpeg::PixelFormat::RGBA,
    };

    turbojpeg::compress(image, quality, turbojpeg::Subsamp::Sub2x2)
        .map(|buf| buf.to_vec())
        .map_err(|e| PipelineError::Encode(format!("{e}")))
}
