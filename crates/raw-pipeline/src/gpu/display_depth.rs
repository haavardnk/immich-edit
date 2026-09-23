use wgpu::TextureFormat;

use crate::frame::{BitDepth, OutputFormat};

pub const DISPLAY_STORE_INJECT: &str = "// DISPLAY_STORE_INJECT";
pub const DISPLAY_LOAD_INJECT: &str = "// DISPLAY_LOAD_INJECT";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DisplayDepth {
    Eight,
    Sixteen,
}

impl DisplayDepth {
    pub fn for_output(format: &OutputFormat) -> Self {
        match format.bit_depth() {
            BitDepth::Eight => Self::Eight,
            BitDepth::Sixteen => Self::Sixteen,
        }
    }

    pub fn format(self) -> TextureFormat {
        match self {
            Self::Eight => TextureFormat::Rgba8Unorm,
            Self::Sixteen => TextureFormat::Rgba16Uint,
        }
    }

    pub fn bytes_per_pixel(self) -> u32 {
        match self {
            Self::Eight => 4,
            Self::Sixteen => 8,
        }
    }

    pub fn store_wgsl(self, binding: u32) -> String {
        store_wgsl(self.format(), binding)
    }

    pub fn load_wgsl(self, binding: u32) -> String {
        match self {
            Self::Eight => format!(
                "@group(0) @binding({binding}) var src_tex: texture_2d<f32>;\n\
                 fn load_display(c: vec2<i32>) -> vec4<f32> {{\n    \
                 return textureLoad(src_tex, c, 0);\n}}"
            ),
            Self::Sixteen => format!(
                "@group(0) @binding({binding}) var src_tex: texture_2d<u32>;\n\
                 fn load_display(c: vec2<i32>) -> vec4<f32> {{\n    \
                 return vec4<f32>(textureLoad(src_tex, c, 0)) / 65535.0;\n}}"
            ),
        }
    }

    pub fn load_u8_wgsl(self, binding: u32) -> String {
        match self {
            Self::Eight => format!(
                "@group(0) @binding({binding}) var src_tex: texture_2d<f32>;\n\
                 fn load_display_u8(c: vec2<i32>) -> vec3<u32> {{\n    \
                 return vec3<u32>(round(textureLoad(src_tex, c, 0).rgb * 255.0));\n}}"
            ),
            Self::Sixteen => format!(
                "@group(0) @binding({binding}) var src_tex: texture_2d<u32>;\n\
                 fn load_display_u8(c: vec2<i32>) -> vec3<u32> {{\n    \
                 return textureLoad(src_tex, c, 0).rgb >> vec3<u32>(8u);\n}}"
            ),
        }
    }
}

pub fn store_wgsl(format: TextureFormat, binding: u32) -> String {
    let decl = |kind: &str| {
        format!("@group(0) @binding({binding}) var out_tex: texture_storage_2d<{kind}, write>;")
    };
    match format {
        TextureFormat::Rgba16Uint => format!(
            "{}\nfn store_display(c: vec2<i32>, v: vec4<f32>) {{\n    \
             let q = clamp(v, vec4<f32>(0.0), vec4<f32>(1.0)) * 65535.0;\n    \
             textureStore(out_tex, c, vec4<u32>(round(q)));\n}}",
            decl("rgba16uint")
        ),
        TextureFormat::Rgba16Float => format!(
            "{}\nfn store_display(c: vec2<i32>, v: vec4<f32>) {{\n    \
             textureStore(out_tex, c, v);\n}}",
            decl("rgba16float")
        ),
        _ => format!(
            "{}\nfn store_display(c: vec2<i32>, v: vec4<f32>) {{\n    \
             textureStore(out_tex, c, v);\n}}",
            decl("rgba8unorm")
        ),
    }
}
