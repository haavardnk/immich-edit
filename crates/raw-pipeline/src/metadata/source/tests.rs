use little_exif::exif_tag::ExifTag;

use super::read;

const EXPECTED: [(&str, Option<&str>); 22] = [
    ("Canon_EOS_5D_3-2.cr2", Some("Canon")),
    ("Canon_EOS_R6_3-2.cr3", Some("Canon")),
    ("cat.hif", Some("SONY")),
    ("Fujifilm_FinePix_S5000_4-3.raf", Some("FUJIFILM")),
    (
        "Fujifilm_X-T2_14bit_14bit_compressed_3-2.raf",
        Some("FUJIFILM"),
    ),
    (
        "Leica_M8_8bit_8bit_uncompressed_3-2.dng",
        Some("Leica Camera AG"),
    ),
    (
        "Nikon_D2H_12bit_12bit_compressed_Lossy_type_1_3-2.nef",
        Some("NIKON CORPORATION"),
    ),
    (
        "Olympus_E-M1MarkII_16bit_4-3.orf",
        Some("OLYMPUS CORPORATION"),
    ),
    ("Panasonic_DMC-LX7_1-1.rw2", Some("Panasonic")),
    ("paris_exif_xmp_icc.tif", Some("Google")),
    ("paris_exif_xmp_icc.webp", Some("Google")),
    ("paris_exif_xmp_icc_boxes.jxl", Some("Google")),
    ("paris_exif_xmp_icc_brob.jxl", Some("Google")),
    ("paris_icc_exif_xmp.avif", Some("Google")),
    ("paris_icc_exif_xmp.png", Some("Google")),
    ("paris_lightroom.jpg", Some("Google")),
    (
        "Pentax_K10D_12bit_12bit_compressed_3-2.pef",
        Some("PENTAX Corporation"),
    ),
    ("pug.heic", Some("Apple")),
    (
        "Ricoh_GR_12bit_4-3.dng",
        Some("RICOH IMAGING COMPANY, LTD."),
    ),
    ("seine_sdr_gainmap_srgb.jpg", Some("Google")),
    ("Sony_ILCE-7S_14bit_14bit_compressed_3-2.arw", Some("SONY")),
    ("xiaomi.heic", Some("Xiaomi")),
];

#[test]
fn reads_exif_from_every_fixture_container() {
    let failures: Vec<String> = EXPECTED
        .iter()
        .filter_map(|&(name, make)| {
            let path = raw_pipeline_testkit::fixtures::fixture_path(name);
            let Ok(bytes) = std::fs::read(&path) else {
                return Some(format!("{name}: missing fixture"));
            };
            let found_make = read(&bytes).and_then(|exif| {
                exif.get_tag(&ExifTag::Make(String::new()))
                    .find_map(|tag| match tag {
                        ExifTag::Make(v) => Some(v.trim_end_matches(['\0', ' ']).to_string()),
                        _ => None,
                    })
            });
            (found_make.as_deref() != make)
                .then(|| format!("{name}: make {found_make:?} (want {make:?})"))
        })
        .collect();
    if !failures.is_empty() {
        panic!("{}", failures.join("\n"));
    }
}
