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

#[cfg(test)]
mod tests {
    use super::*;

    fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack.windows(needle.len()).position(|w| w == needle)
    }

    fn riff_size(buf: &[u8]) -> u32 {
        u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]])
    }

    fn make_vp8_webp(w: u16, h: u16) -> Vec<u8> {
        let mut frame: Vec<u8> = vec![0, 0, 0, 0x9D, 0x01, 0x2A];
        frame.extend_from_slice(&w.to_le_bytes());
        frame.extend_from_slice(&h.to_le_bytes());
        while frame.len() < 16 {
            frame.push(0);
        }
        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(b"RIFF");
        let riff_size = (4 + 8 + frame.len()) as u32;
        out.extend_from_slice(&riff_size.to_le_bytes());
        out.extend_from_slice(b"WEBP");
        out.extend_from_slice(b"VP8 ");
        out.extend_from_slice(&(frame.len() as u32).to_le_bytes());
        out.extend_from_slice(&frame);
        out
    }

    fn make_vp8l_webp(w_m1: u32, h_m1: u32) -> Vec<u8> {
        let b0 = (w_m1 & 0xFF) as u8;
        let b1 = (((w_m1 >> 8) & 0x3F) | ((h_m1 & 0x03) << 6)) as u8;
        let b2 = ((h_m1 >> 2) & 0xFF) as u8;
        let b3 = ((h_m1 >> 10) & 0x0F) as u8;
        let payload: Vec<u8> = vec![0x2F, b0, b1, b2, b3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(b"RIFF");
        let riff_size = (4 + 8 + payload.len()) as u32;
        out.extend_from_slice(&riff_size.to_le_bytes());
        out.extend_from_slice(b"WEBP");
        out.extend_from_slice(b"VP8L");
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&payload);
        out
    }

    fn make_vp8x_webp(w_m1: u32, h_m1: u32) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(b"RIFF");
        let body_size: u32 = 4 + 8 + 10;
        out.extend_from_slice(&body_size.to_le_bytes());
        out.extend_from_slice(b"WEBP");
        out.extend_from_slice(b"VP8X");
        out.extend_from_slice(&10u32.to_le_bytes());
        out.push(0);
        out.extend_from_slice(&[0, 0, 0]);
        out.extend_from_slice(&w_m1.to_le_bytes()[0..3]);
        out.extend_from_slice(&h_m1.to_le_bytes()[0..3]);
        out
    }

    #[test]
    fn jpeg_unchanged_when_invalid() {
        for input in [vec![], vec![0xFF], vec![0x00, 0x00], vec![0xFF, 0xD9, 0x12]] {
            let out = embed_jpeg_icc(input.clone());
            assert_eq!(out, input);
        }
    }

    #[test]
    fn jpeg_embeds_app2_icc_profile() {
        let jpeg: Vec<u8> = vec![0xFF, 0xD8, 0xFF, 0xD9];
        let out = embed_jpeg_icc(jpeg);
        assert_eq!(&out[0..2], &[0xFF, 0xD8]);
        let marker_pos = find_subslice(&out, b"ICC_PROFILE\0").expect("ICC marker present");
        assert_eq!(out[marker_pos - 4], 0xFF);
        assert_eq!(out[marker_pos - 3], 0xE2);
        let seg_len = u16::from_be_bytes([out[marker_pos - 2], out[marker_pos - 1]]) as usize;
        assert_eq!(seg_len, 2 + 12 + 2 + SRGB_ICC.len());
        let chunk_no = out[marker_pos + 12];
        let chunk_total = out[marker_pos + 13];
        assert_eq!(chunk_no, 1);
        assert_eq!(chunk_total, 1);
    }

    #[test]
    fn webp_unchanged_when_invalid() {
        for input in [
            vec![],
            vec![0u8; 16],
            {
                let mut bad = make_vp8_webp(100, 80);
                bad[0] = b'X';
                bad
            },
            {
                let mut bad = make_vp8_webp(100, 80);
                bad[8] = b'X';
                bad
            },
        ] {
            let out = embed_webp_icc(input.clone());
            assert_eq!(out, input);
        }
    }

    #[test]
    fn webp_vp8_and_vp8l_wrap_into_vp8x_with_iccp() {
        for input in [make_vp8_webp(100, 80), make_vp8l_webp(99, 79)] {
            let out = embed_webp_icc(input);
            assert_eq!(&out[0..4], b"RIFF");
            assert_eq!(&out[8..12], b"WEBP");
            assert_eq!(&out[12..16], b"VP8X");
            assert_eq!(riff_size(&out) as usize, out.len() - 8);
            assert_eq!(out[20] & 0b0010_0000, 0b0010_0000);
            let w_m1 = u32::from_le_bytes([out[24], out[25], out[26], 0]);
            let h_m1 = u32::from_le_bytes([out[27], out[28], out[29], 0]);
            assert_eq!(w_m1, 99);
            assert_eq!(h_m1, 79);
            let iccp = find_subslice(&out, b"ICCP").expect("ICCP chunk present");
            let icc_size =
                u32::from_le_bytes([out[iccp + 4], out[iccp + 5], out[iccp + 6], out[iccp + 7]])
                    as usize;
            assert_eq!(icc_size, SRGB_ICC.len());
            assert_eq!(&out[iccp + 8..iccp + 8 + icc_size], SRGB_ICC);
        }
    }

    #[test]
    fn webp_vp8x_splices_iccp_and_updates_riff_size() {
        let input = make_vp8x_webp(123, 45);
        let original_len = input.len();
        let out = embed_webp_icc(input);
        assert!(out.len() > original_len);
        assert_eq!(riff_size(&out) as usize, out.len() - 8);
        assert_eq!(out[20] & 0b0010_0000, 0b0010_0000);
        let iccp = find_subslice(&out, b"ICCP").expect("ICCP chunk inserted");
        assert_eq!(iccp, 12 + 8 + 10);
        let icc_size =
            u32::from_le_bytes([out[iccp + 4], out[iccp + 5], out[iccp + 6], out[iccp + 7]])
                as usize;
        assert_eq!(icc_size, SRGB_ICC.len());
        assert_eq!(&out[iccp + 8..iccp + 8 + icc_size], SRGB_ICC);
    }
}
