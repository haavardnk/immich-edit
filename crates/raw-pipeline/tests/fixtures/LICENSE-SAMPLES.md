# Test Fixtures

The fixtures come from two kinds of source: CC0 RAW files and BSD-licensed phone and editor
images. Each section below gives the licence and the upstream path of every file. Files marked
"derived" were changed from their upstream file by the exact command shown.

## RAW samples (CC0)

RAW samples curated from <https://raw.pixls.us/>, smallest CC0 file per mainstream brand that rawler can decode.

All files released under CC0 1.0 Public Domain Dedication: <https://creativecommons.org/publicdomain/zero/1.0/>

The X-T2 is the X-Trans sample. It shoots a grey step chart, so `cpu_render::xtrans_renders_neutral_greys` can check that neutral patches stay neutral.

| File | Camera | Mode | Size | Source |
|------|--------|------|------|--------|
| `Canon_EOS_5D_3-2.cr2` | Canon EOS 5D | RAW (3:2) | 13 MB | <https://raw.pixls.us/getfile.php/1346/nice/Canon - EOS 5D - RAW (3:2).CR2> |
| `Canon_EOS_R6_3-2.cr3` | Canon EOS R6 | 3:2 | 5.0 MB | <https://raw.pixls.us/getfile.php/4659/nice/Canon - EOS R6 - 3:2.CR3> |
| `Fujifilm_FinePix_S5000_4-3.raf` | Fujifilm FinePix S5000 | 4:3 | 6.5 MB | <https://raw.pixls.us/getfile.php/2726/nice/Fujifilm - FinePix S5000 - 4:3.RAF> |
| `Fujifilm_X-T2_14bit_14bit_compressed_3-2.raf` | Fujifilm X-T2 | 14bit 14bit compressed (3:2) | 23 MB | <https://raw.pixls.us/getfile.php/865/nice/Fujifilm - X-T2 - 14bit 14bit compressed (3:2).RAF> |
| `Leica_M8_8bit_8bit_uncompressed_3-2.dng` | Leica M8 | 8bit 8bit uncompressed (3:2) | 10.1 MB | <https://raw.pixls.us/getfile.php/2988/nice/Leica - M8 - 8bit 8bit uncompressed (3:2).DNG> |
| `Nikon_D2H_12bit_12bit_compressed_Lossy_type_1_3-2.nef` | Nikon D2H | 12bit 12bit compressed (Lossy (type 1)) (3:2) | 3.1 MB | <https://raw.pixls.us/getfile.php/5227/nice/Nikon - D2H - 12bit 12bit compressed (Lossy (type 1)) (3:2).NEF> |
| `Olympus_E-M1MarkII_16bit_4-3.orf` | Olympus E-M1 Mark II | 16bit (4:3) | 17 MB | <https://raw.pixls.us/getfile.php/1993/nice/Olympus - E-M1MarkII - 16bit (4:3).ORF> |
| `Panasonic_DMC-LX7_1-1.rw2` | Panasonic DMC-LX7 | 1:1 | 3.1 MB | <https://raw.pixls.us/getfile.php/7008/nice/Panasonic - DMC-LX7 - 1:1.RW2> |
| `Pentax_K10D_12bit_12bit_compressed_3-2.pef` | Pentax K10D | 12bit 12bit compressed (3:2) | 9.1 MB | <https://raw.pixls.us/getfile.php/2239/nice/Pentax - K10D - 12bit 12bit compressed (3:2).PEF> |
| `Ricoh_GR_12bit_4-3.dng` | Ricoh GR | 12bit (4:3) | 11.6 MB | <https://raw.pixls.us/getfile.php/996/nice/Ricoh - GR - 12bit (4:3).DNG> |
| `Sony_ILCE-7S_14bit_14bit_compressed_3-2.arw` | Sony ILCE-7S | 14bit 14bit compressed (3:2) | 5.9 MB | <https://raw.pixls.us/getfile.php/1582/nice/Sony - ILCE-7S - 14bit 14bit compressed (3:2).ARW> |

## Phone HEIF samples (BSD-3-Clause, pillow_heif)

Unchanged copies from <https://github.com/bigcat88/pillow_heif>. They cover Apple, Android and
Sony HEIF files with maker notes, GPS and HEIF brands little_exif does not detect.

| File | Camera | Notes | Size | Source |
|------|--------|-------|------|--------|
| `pug.heic` | Apple iPhone 13 Pro | Apple maker note, depth map, HDR gain map XMP, GPS | 1.3 MB | `tests/images/heif_other/pug.heic` |
| `xiaomi.heic` | Xiaomi M2012K11AG | Android, GPS | 0.9 MB | `tests/images/heif_special/xiaomi.heic` |
| `cat.hif` | Sony ILCE-7SM3 | `heix` major brand, 10-bit, Sony maker note, XMP | 2.6 MB | `tests/images/heif_other/cat.hif` |

```text
Copyright (c) 2021-2023, Pillow-Heif contributors.
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are
met:

    * Redistributions of source code must retain the above copyright
       notice, this list of conditions and the following disclaimer.

    * Redistributions in binary form must reproduce the above
       copyright notice, this list of conditions and the following
       disclaimer in the documentation and/or other materials provided
       with the distribution.

    * Neither the name of the Pillow-Heif nor the names of any
       contributors may be used to endorse or promote products derived
       from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
```

## Editor and container samples (BSD-2-Clause, libavif)

From <https://github.com/AOMediaCodec/libavif> `tests/data/`, licensed "same as libavif". The
derived files start from `tests/data/paris_exif_xmp_icc.jpg` (Pixel 4a, saved by GIMP) and
keep its licence. Commands use exiftool 13.55, cwebp 1.6.0, cjxl 0.12.0 and macOS `sips`.

| File | Content | Size | Source |
|------|---------|------|--------|
| `seine_sdr_gainmap_srgb.jpg` | Pixel 4a through Camera Raw 15.5: `crs:` settings, IPTC, Ultra HDR gain map, GCamera, GPS | 140 KB | `tests/data/seine_sdr_gainmap_srgb.jpg` |
| `paris_icc_exif_xmp.png` | EXIF and XMP in PNG | 150 KB | `tests/data/paris_icc_exif_xmp.png` |
| `paris_icc_exif_xmp.avif` | EXIF and XMP in AVIF | 21 KB | `tests/data/paris_icc_exif_xmp.avif` |
| `paris_exif_xmp_icc.webp` | Derived: `cwebp -quiet -q 80 -metadata all paris_exif_xmp_icc.jpg -o paris_exif_xmp_icc.webp` | 24 KB | `tests/data/paris_exif_xmp_icc.jpg` |
| `paris_exif_xmp_icc_boxes.jxl` | Derived: `cjxl --quiet --compress_boxes=0 paris_exif_xmp_icc.jpg paris_exif_xmp_icc_boxes.jxl` (plain `Exif` and `xml ` boxes) | 17 KB | `tests/data/paris_exif_xmp_icc.jpg` |
| `paris_exif_xmp_icc_brob.jxl` | Derived: `cjxl --quiet paris_exif_xmp_icc.jpg paris_exif_xmp_icc_brob.jxl` (Brotli `brob` boxes) | 14 KB | `tests/data/paris_exif_xmp_icc.jpg` |
| `paris_exif_xmp_icc.tif` | Derived: `sips -s format tiff paris_exif_xmp_icc.jpg --out paris_exif_xmp_icc.tif`, then `exiftool -overwrite_original -tagsfromfile paris_exif_xmp_icc.jpg -all:all paris_exif_xmp_icc.tif` | 360 KB | `tests/data/paris_exif_xmp_icc.jpg` |
| `paris_lightroom.jpg` | Derived Lightroom-style XMP, see below | 22 KB | `tests/data/paris_exif_xmp_icc.jpg` |

`paris_lightroom.jpg` adds a rating, keywords, a caption, a place, XMP GPS, a motion photo flag,
Camera Raw settings and an XMP orientation that disagrees with EXIF:

```sh
exiftool -o paris_lightroom.jpg -XMP-xmp:Rating=4 -XMP-dc:Subject=Paris -XMP-dc:Subject=Seine \
  '-XMP-lr:HierarchicalSubject=Places|France|Paris' '-XMP-dc:Description=Evening on the Seine' \
  -XMP-photoshop:City=Paris -XMP-GCamera:MotionPhoto=1 -XMP-crs:ProcessVersion=15.4 \
  -XMP-crs:Exposure2012=0.5 paris_exif_xmp_icc.jpg
exiftool -overwrite_original '-XMP-exif:GPSLatitude=48.8566 N' '-XMP-exif:GPSLongitude=2.3522 E' \
  '-XMP-tiff:Orientation#=6' paris_lightroom.jpg
```

```text
Copyright 2019 Joe Drago. All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
this list of conditions and the following disclaimer in the documentation
and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
```
