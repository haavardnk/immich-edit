#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelKind {
    Subject,
    People,
    Sky,
    Depth,
    Semantic,
    Click,
}

impl ModelKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Subject => "subject",
            Self::People => "people",
            Self::Sky => "sky",
            Self::Depth => "depth",
            Self::Semantic => "semantic",
            Self::Click => "click",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Nchw,
    Nhwc,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Normalization {
    pub scale: f32,
    pub mean: [f32; 3],
    pub std: [f32; 3],
}

impl Normalization {
    pub const IMAGENET: Self = Self {
        scale: 1.0 / 255.0,
        mean: [0.485, 0.456, 0.406],
        std: [0.229, 0.224, 0.225],
    };
    pub const SIGNED: Self = Self {
        scale: 1.0 / 255.0,
        mean: [0.5, 0.5, 0.5],
        std: [0.5, 0.5, 0.5],
    };
    pub const HALF_MEAN: Self = Self {
        scale: 1.0 / 255.0,
        mean: [0.5, 0.5, 0.5],
        std: [1.0, 1.0, 1.0],
    };
    pub const UNIT: Self = Self {
        scale: 1.0 / 255.0,
        mean: [0.0, 0.0, 0.0],
        std: [1.0, 1.0, 1.0],
    };
    pub const MMSEG: Self = Self {
        scale: 1.0,
        mean: [123.675, 116.28, 103.53],
        std: [58.395, 57.12, 57.375],
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    Stretch,
    Contain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activation {
    Sigmoid,
    MinMax,
    Softmax { channel: usize },
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelSpec {
    pub kind: ModelKind,
    pub input_edge: u32,
    pub layout: Layout,
    pub fit: Fit,
    pub normalization: Normalization,
    pub activation: Activation,
}

impl ModelSpec {
    pub const U2NET: Self = Self {
        kind: ModelKind::Subject,
        input_edge: 320,
        layout: Layout::Nchw,
        fit: Fit::Contain,
        normalization: Normalization::IMAGENET,
        activation: Activation::MinMax,
    };
    pub const ORMBG: Self = Self {
        kind: ModelKind::Subject,
        input_edge: 1024,
        layout: Layout::Nchw,
        fit: Fit::Stretch,
        normalization: Normalization::UNIT,
        activation: Activation::MinMax,
    };
    pub const MODNET: Self = Self {
        kind: ModelKind::People,
        input_edge: 512,
        layout: Layout::Nchw,
        fit: Fit::Contain,
        normalization: Normalization::SIGNED,
        activation: Activation::None,
    };
    pub const SKYSEG: Self = Self {
        kind: ModelKind::Sky,
        input_edge: 320,
        layout: Layout::Nchw,
        fit: Fit::Contain,
        normalization: Normalization::IMAGENET,
        activation: Activation::MinMax,
    };
    pub const DEPTH_ANYTHING: Self = Self {
        kind: ModelKind::Depth,
        input_edge: 518,
        layout: Layout::Nchw,
        fit: Fit::Contain,
        normalization: Normalization::IMAGENET,
        activation: Activation::MinMax,
    };
    pub const MOBILE_SAM: Self = Self {
        kind: ModelKind::Click,
        input_edge: 1024,
        layout: Layout::Nchw,
        fit: Fit::Contain,
        normalization: Normalization::IMAGENET,
        activation: Activation::Sigmoid,
    };
    pub const SEGFORMER_ADE: Self = Self {
        kind: ModelKind::Semantic,
        input_edge: 512,
        layout: Layout::Nchw,
        fit: Fit::Stretch,
        normalization: Normalization::IMAGENET,
        activation: Activation::Softmax { channel: 0 },
    };
}
