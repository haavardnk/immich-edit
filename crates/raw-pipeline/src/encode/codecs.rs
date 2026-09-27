use libheif_rs::{CompressionFormat, LibHeif};

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct HeifCodecs {
    pub hevc_decode: bool,
    pub hevc_encode: bool,
    pub av1_decode: bool,
    pub av1_encode: bool,
}

pub fn heif_codecs() -> HeifCodecs {
    let lib = LibHeif::new();
    HeifCodecs {
        hevc_decode: !lib
            .decoder_descriptors(1, Some(CompressionFormat::Hevc))
            .is_empty(),
        hevc_encode: !lib
            .encoder_descriptors(1, Some(CompressionFormat::Hevc), None)
            .is_empty(),
        av1_decode: !lib
            .decoder_descriptors(1, Some(CompressionFormat::Av1))
            .is_empty(),
        av1_encode: !lib
            .encoder_descriptors(1, Some(CompressionFormat::Av1), None)
            .is_empty(),
    }
}
