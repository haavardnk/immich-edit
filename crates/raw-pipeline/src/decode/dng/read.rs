use rawler::formats::tiff::reader::TiffReader;
use rawler::formats::tiff::{GenericTiffReader, IFD, Value};
use rawler::tags::{DngTag, TiffCommonTag};

const RAW_COMPRESSIONS: [u32; 5] = [1, 7, 8, 0x884c, 52546];

fn is_tiff(data: &[u8]) -> bool {
    data.starts_with(b"II*\0") || data.starts_with(b"MM\0*")
}

fn raw_ifd(tiff: &GenericTiffReader) -> Option<&IFD> {
    tiff.find_ifds_with_tag(TiffCommonTag::Compression)
        .into_iter()
        .find(|ifd| {
            let compression = ifd
                .get_entry(TiffCommonTag::Compression)
                .map(|e| e.force_u32(0));
            let reduced = ifd
                .get_entry(TiffCommonTag::NewSubFileType)
                .is_some_and(|e| e.force_u32(0) & 1 != 0);
            !reduced && compression.is_some_and(|c| RAW_COMPRESSIONS.contains(&c))
        })
}

fn blob(ifd: &IFD, tag: DngTag) -> Vec<u8> {
    match ifd.get_entry(tag).map(|e| &e.value) {
        Some(Value::Undefined(bytes) | Value::Byte(bytes)) => bytes.clone(),
        _ => Vec::new(),
    }
}

pub(super) fn opcode_blobs(data: &[u8]) -> Option<[Vec<u8>; 3]> {
    if !is_tiff(data) {
        return None;
    }
    let tiff = GenericTiffReader::new_with_buffer(data, 0, 0, None).ok()?;
    tiff.find_first_ifd_with_tag(DngTag::DNGVersion)?;
    let raw = raw_ifd(&tiff)?;
    Some(
        [
            DngTag::OpcodeList1,
            DngTag::OpcodeList2,
            DngTag::OpcodeList3,
        ]
        .map(|tag| blob(raw, tag)),
    )
}
