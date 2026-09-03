use axum::Json;
use axum::extract::Path;
use serde::Serialize;

use crate::asset_key::AssetKey;
use crate::error::AppError;
use crate::immich::ImmichError;
use crate::immich::dto::AssetFace;
use crate::routes::auth::AuthCtx;

#[derive(Debug, Clone, Serialize)]
pub struct FaceBox {
    pub source_w: u32,
    pub source_h: u32,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub async fn list(ctx: AuthCtx, Path(id): Path<AssetKey>) -> Result<Json<Vec<FaceBox>>, AppError> {
    let faces = match ctx.immich.asset_faces(id.source()).await {
        Ok(faces) => faces,
        Err(ImmichError::NotFound) => Vec::new(),
        Err(err) => return Err(err.into()),
    };
    Ok(Json(faces.iter().filter_map(normalize).collect()))
}

fn normalize(face: &AssetFace) -> Option<FaceBox> {
    if face.image_width == 0 || face.image_height == 0 {
        return None;
    }
    let sw = f64::from(face.image_width);
    let sh = f64::from(face.image_height);
    let x0 = (face.bounding_box_x1.min(face.bounding_box_x2) as f64 / sw).clamp(0.0, 1.0);
    let x1 = (face.bounding_box_x1.max(face.bounding_box_x2) as f64 / sw).clamp(0.0, 1.0);
    let y0 = (face.bounding_box_y1.min(face.bounding_box_y2) as f64 / sh).clamp(0.0, 1.0);
    let y1 = (face.bounding_box_y1.max(face.bounding_box_y2) as f64 / sh).clamp(0.0, 1.0);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some(FaceBox {
        source_w: face.image_width,
        source_h: face.image_height,
        x: x0 as f32,
        y: y0 as f32,
        w: (x1 - x0) as f32,
        h: (y1 - y0) as f32,
    })
}
