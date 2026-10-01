pub(crate) mod atrous;
pub(crate) mod chroma;
pub(crate) mod estimate;
pub(crate) mod luma;
pub(crate) mod shrink;

pub(crate) const LUMA_LEVELS: usize = 4;
pub(crate) const CHROMA_LEVELS: usize = 5;
pub(crate) const PB_DEN: f32 = 1.8556;
pub(crate) const PR_DEN: f32 = 1.5748;

#[cfg(test)]
mod tests;
