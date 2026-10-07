use std::io::{Cursor, Seek, Write};

use little_exif::endian::Endian;
use little_exif::exif_tag::ExifTag;
use little_exif::ifd::ExifTagGroup;
use tiff::Directory;
use tiff::encoder::colortype::{self, ColorType};
use tiff::encoder::{
    Compression, DeflateLevel, DirectoryEncoder, TiffEncoder, TiffKindStandard, TiffValue,
};
use tiff::tags::{ByteOrder, Tag, Type};

use super::ImageRgb8;
use crate::PipelineError;
use crate::frame::{OutputColorSpace, TiffCompression};
use crate::metadata::exif::ExifBlock;

const POINTER_TAGS: [u16; 3] = [0x8769, 0x8825, 0xa005];

pub fn encode_tiff8(
    img: ImageRgb8<'_>,
    compression: TiffCompression,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    write::<colortype::RGB8>(img.width, img.height, img.rgb, compression, cs, exif)
}

pub fn encode_tiff16(
    rgb16: &[u16],
    width: u32,
    height: u32,
    compression: TiffCompression,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    write::<colortype::RGB16>(width, height, rgb16, compression, cs, exif)
}

fn tiff_err(e: tiff::TiffError) -> PipelineError {
    PipelineError::Encode(format!("tiff: {e}"))
}

fn write<C: ColorType>(
    width: u32,
    height: u32,
    data: &[C::Inner],
    compression: TiffCompression,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>>
where
    [C::Inner]: TiffValue,
{
    let mut buf: Vec<u8> = Vec::new();
    let mut enc = TiffEncoder::new(Cursor::new(&mut buf))
        .map_err(tiff_err)?
        .with_compression(match compression {
            TiffCompression::None => Compression::Uncompressed,
            TiffCompression::Lzw => Compression::Lzw,
            TiffCompression::Deflate => Compression::Deflate(DeflateLevel::default()),
        });
    let exif_ifd = exif
        .map(|e| sub_directory(&mut enc, e, ExifTagGroup::EXIF))
        .transpose()?
        .flatten();
    let gps_ifd = exif
        .map(|e| sub_directory(&mut enc, e, ExifTagGroup::GPS))
        .transpose()?
        .flatten();
    let mut image = enc.new_image::<C>(width, height).map_err(tiff_err)?;
    let dir = image.encoder();
    dir.write_tag(Tag::IccProfile, cs.icc_profile())
        .map_err(tiff_err)?;
    if let Some(exif) = exif {
        let entries = entries(dir, exif, ExifTagGroup::GENERIC)?;
        dir.extend_from(&entries);
    }
    if let Some(offset) = exif_ifd {
        dir.write_tag(Tag::ExifDirectory, offset)
            .map_err(tiff_err)?;
    }
    if let Some(offset) = gps_ifd {
        dir.write_tag(Tag::GpsDirectory, offset).map_err(tiff_err)?;
    }
    image.write_data(data).map_err(tiff_err)?;
    Ok(buf)
}

fn sub_directory<W: Write + Seek>(
    enc: &mut TiffEncoder<W>,
    exif: &ExifBlock,
    group: ExifTagGroup,
) -> crate::PipelineResult<Option<u32>> {
    if tags(exif, group).next().is_none() {
        return Ok(None);
    }
    let mut dir = enc.extra_directory().map_err(tiff_err)?;
    let entries = entries(&mut dir, exif, group)?;
    dir.extend_from(&entries);
    Ok(Some(dir.finish_with_offsets().map_err(tiff_err)?.offset))
}

fn tags(exif: &ExifBlock, group: ExifTagGroup) -> impl Iterator<Item = &ExifTag> {
    exif.tags
        .into_iter()
        .filter(move |t| t.get_group() == group && !POINTER_TAGS.contains(&t.as_u16()))
}

fn entries<W: Write + Seek>(
    dir: &mut DirectoryEncoder<'_, W, TiffKindStandard>,
    exif: &ExifBlock,
    group: ExifTagGroup,
) -> crate::PipelineResult<Directory> {
    let endian = match ByteOrder::native() {
        ByteOrder::LittleEndian => Endian::Little,
        ByteOrder::BigEndian => Endian::Big,
    };
    tags(exif, group)
        .filter_map(|t| Some((t, Type::from_u16(t.format().as_u16())?)))
        .map(|(t, ty)| {
            let entry = dir
                .write_entry_bytes(ty, &value(t, &endian))
                .map_err(tiff_err)?;
            Ok((Tag::from_u16_exhaustive(t.as_u16()), entry))
        })
        .collect()
}

fn value(tag: &ExifTag, endian: &Endian) -> Vec<u8> {
    let mut bytes = tag.value_as_u8_vec(endian);
    if tag.is_string() {
        bytes.resize(tag.number_of_components() as usize, 0);
    }
    bytes
}
