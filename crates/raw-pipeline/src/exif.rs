use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;

use crate::PipelineError;

pub fn parse(data: &[u8]) -> Option<Metadata> {
    let ext = detect(data)?;
    let vec = data.to_vec();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        Metadata::new_from_vec(&vec, ext).ok()
    }))
    .ok()
    .flatten()
}

fn detect(data: &[u8]) -> Option<FileExtension> {
    let mut cursor = std::io::Cursor::new(data);
    FileExtension::auto_detect(&mut cursor)
}

pub fn inject_jpeg(jpeg: &mut Vec<u8>, exif: &Metadata) -> crate::PipelineResult<()> {
    let mut m = exif.clone();
    m.set_tag(ExifTag::Orientation(vec![1]));
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        m.write_to_vec(jpeg, FileExtension::JPEG)
    }))
    .map_err(|_| PipelineError::Encode("exif write panic".into()))?
    .map_err(|e| PipelineError::Encode(format!("exif: {e}")))
}
