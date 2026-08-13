use raw_pipeline::frame::{OutputFormat, RenderOptions};
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::ImmichClient;
use crate::services::render::RenderIdentity;
use crate::state::AppState;

pub struct SceneImage {
    pub rgb8: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub async fn render_scene(
    state: &AppState,
    identity: RenderIdentity,
    immich: ImmichClient,
    owner: Uuid,
    asset_id: AssetKey,
) -> Result<SceneImage, AppError> {
    let mut edits = state.edits.get_edits_or_default(owner, asset_id).await?;
    edits.geometry = Default::default();
    edits.lens = raw_pipeline::edits::LensEdits {
        profile_enabled: Some(false),
        ..Default::default()
    };
    edits.effects = Default::default();
    edits.masks.clear();

    let opts = RenderOptions {
        max_edge: state.segment.max_edge(),
        output: OutputFormat::Rgb8,
        ..Default::default()
    };
    let rendered = state
        .render
        .render(identity, immich, asset_id.source(), edits, opts, None)
        .await?;
    Ok(SceneImage {
        rgb8: rendered.bytes,
        width: rendered.width,
        height: rendered.height,
    })
}

pub fn combine_coverage(base: &[u8], patch: &[u8], subtract: bool) -> Vec<u8> {
    base.iter()
        .zip(patch)
        .map(|(b, p)| {
            if subtract {
                ((*b as u16 * (255 - *p) as u16 + 127) / 255) as u8
            } else {
                (*b).max(*p)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::combine_coverage;

    #[test]
    fn adding_keeps_the_strongest_coverage() {
        let out = combine_coverage(&[0, 128, 255], &[64, 32, 0], false);
        assert_eq!(out, vec![64, 128, 255]);
    }

    #[test]
    fn subtracting_carves_the_patch_out() {
        let out = combine_coverage(&[255, 255, 128], &[255, 0, 128], true);
        assert_eq!(out, vec![0, 255, 64]);
    }
}
