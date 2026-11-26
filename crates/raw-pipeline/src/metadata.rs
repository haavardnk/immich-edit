pub mod exif;
mod source;

use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;

pub use source::read;

#[derive(Debug, Clone)]
pub struct ExportMetadata {
    pub exif: Option<Metadata>,
    pub location: bool,
    pub overrides: Vec<ExifTag>,
    pub cleared: Vec<u16>,
}
