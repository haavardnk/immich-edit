pub const SRGB_ICC: &[u8] = include_bytes!("../../assets/icc/sRGB-v2-micro.icc");

pub fn embed_jpeg_icc(jpeg: Vec<u8>) -> Vec<u8> {
    if jpeg.len() < 2 || jpeg[0] != 0xFF || jpeg[1] != 0xD8 {
        return jpeg;
    }
    let icc = SRGB_ICC;
    let chunk_payload_max: usize = 65519;
    let total_chunks = icc.len().div_ceil(chunk_payload_max).max(1);
    if total_chunks > 255 {
        return jpeg;
    }
    let mut out: Vec<u8> = Vec::with_capacity(jpeg.len() + icc.len() + 32);
    out.extend_from_slice(&jpeg[0..2]);
    for chunk_no in 1..=total_chunks {
        let start = (chunk_no - 1) * chunk_payload_max;
        let end = (start + chunk_payload_max).min(icc.len());
        let payload = &icc[start..end];
        let seg_len = 2 + 12 + 2 + payload.len();
        out.push(0xFF);
        out.push(0xE2);
        out.extend_from_slice(&(seg_len as u16).to_be_bytes());
        out.extend_from_slice(b"ICC_PROFILE\0");
        out.push(chunk_no as u8);
        out.push(total_chunks as u8);
        out.extend_from_slice(payload);
    }
    out.extend_from_slice(&jpeg[2..]);
    out
}

pub fn embed_webp_icc(webp: Vec<u8>) -> Vec<u8> {
    if webp.len() < 30 || &webp[0..4] != b"RIFF" || &webp[8..12] != b"WEBP" {
        return webp;
    }
    let icc = SRGB_ICC;
    let fourcc = &webp[12..16];
    let payload_size = u32::from_le_bytes([webp[16], webp[17], webp[18], webp[19]]) as usize;
    let chunk_total = 8 + payload_size + (payload_size & 1);
    if 12 + chunk_total > webp.len() {
        return webp;
    }
    let (width_m1, height_m1) = match fourcc {
        b"VP8 " => {
            let start = 12 + 8;
            if start + 10 > webp.len() {
                return webp;
            }
            let frame = &webp[start..];
            if frame[3] != 0x9D || frame[4] != 0x01 || frame[5] != 0x2A {
                return webp;
            }
            let w = u16::from_le_bytes([frame[6], frame[7]]) & 0x3FFF;
            let h = u16::from_le_bytes([frame[8], frame[9]]) & 0x3FFF;
            (w as u32 - 1, h as u32 - 1)
        }
        b"VP8L" => {
            let start = 12 + 8;
            if start + 5 > webp.len() || webp[start] != 0x2F {
                return webp;
            }
            let b0 = webp[start + 1] as u32;
            let b1 = webp[start + 2] as u32;
            let b2 = webp[start + 3] as u32;
            let b3 = webp[start + 4] as u32;
            let w_m1 = b0 | ((b1 & 0x3F) << 8);
            let h_m1 = ((b1 >> 6) & 0x03) | (b2 << 2) | ((b3 & 0x0F) << 10);
            (w_m1, h_m1)
        }
        b"VP8X" => return splice_iccp_into_vp8x(webp, icc),
        _ => return webp,
    };
    build_extended_webp(width_m1, height_m1, icc, &webp[12..12 + chunk_total])
}

fn build_extended_webp(width_m1: u32, height_m1: u32, icc: &[u8], image_chunk: &[u8]) -> Vec<u8> {
    let icc_chunk_size = 8 + icc.len() + (icc.len() & 1);
    let vp8x_chunk_size = 8 + 10;
    let body_size = vp8x_chunk_size + icc_chunk_size + image_chunk.len();
    let riff_size = 4 + body_size;
    let mut out: Vec<u8> = Vec::with_capacity(8 + riff_size);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(riff_size as u32).to_le_bytes());
    out.extend_from_slice(b"WEBP");
    out.extend_from_slice(b"VP8X");
    out.extend_from_slice(&10u32.to_le_bytes());
    out.push(0b0010_0000);
    out.extend_from_slice(&[0, 0, 0]);
    out.extend_from_slice(&width_m1.to_le_bytes()[0..3]);
    out.extend_from_slice(&height_m1.to_le_bytes()[0..3]);
    out.extend_from_slice(b"ICCP");
    out.extend_from_slice(&(icc.len() as u32).to_le_bytes());
    out.extend_from_slice(icc);
    if icc.len() & 1 == 1 {
        out.push(0);
    }
    out.extend_from_slice(image_chunk);
    out
}

fn splice_iccp_into_vp8x(mut webp: Vec<u8>, icc: &[u8]) -> Vec<u8> {
    if webp.len() < 30 {
        return webp;
    }
    webp[20] |= 0b0010_0000;
    let icc_chunk_size = 8 + icc.len() + (icc.len() & 1);
    let mut icc_chunk: Vec<u8> = Vec::with_capacity(icc_chunk_size);
    icc_chunk.extend_from_slice(b"ICCP");
    icc_chunk.extend_from_slice(&(icc.len() as u32).to_le_bytes());
    icc_chunk.extend_from_slice(icc);
    if icc.len() & 1 == 1 {
        icc_chunk.push(0);
    }
    let insert_at = 12 + 8 + 10;
    if insert_at > webp.len() {
        return webp;
    }
    let mut out: Vec<u8> = Vec::with_capacity(webp.len() + icc_chunk.len());
    out.extend_from_slice(&webp[0..insert_at]);
    out.extend_from_slice(&icc_chunk);
    out.extend_from_slice(&webp[insert_at..]);
    let new_riff_size = (out.len() - 8) as u32;
    out[4..8].copy_from_slice(&new_riff_size.to_le_bytes());
    out
}
