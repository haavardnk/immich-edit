use std::sync::Arc;

use raw_pipeline::edits::Edits;
use raw_pipeline::frame::RawFrame;
use raw_pipeline::mask_raster::{MaskRaster, RasterMap};

use crate::services::dcp_store::DcpStore;
use crate::services::lut_store::LutStore;
use crate::services::raster_store::RasterStore;

use super::{RenderError, RenderIdentity};

pub struct DcpSelection {
    pub id: String,
    pub profile: Arc<raw_pipeline::dcp::DcpProfile>,
}

#[derive(Clone)]
pub struct RenderInputs {
    rasters: RasterStore,
    luts: LutStore,
    dcp: DcpStore,
}

impl RenderInputs {
    pub fn new(rasters: RasterStore, luts: LutStore, dcp: DcpStore) -> Self {
        Self { rasters, luts, dcp }
    }

    pub async fn dcp_revision(&self) -> Result<String, RenderError> {
        self.dcp
            .revision()
            .await
            .map_err(|e| RenderError::Dcp(e.to_string()))
    }

    pub async fn rasters_for(&self, identity: RenderIdentity, edits: &Edits) -> RasterMap {
        let ids = edits.referenced_raster_ids();
        let mut map: RasterMap = RasterMap::with_capacity(ids.len());
        for id in ids {
            match self
                .rasters
                .load(identity.server_epoch, identity.owner, &id)
                .await
            {
                Ok((meta, bytes)) => {
                    if let Some(r) = MaskRaster::new(meta.width, meta.height, bytes) {
                        map.insert(id, Arc::new(r));
                    }
                }
                Err(e) => {
                    tracing::warn!(raster_id = %id, error = %e, "raster load failed");
                }
            }
        }
        map
    }

    pub async fn luts_for(&self, edits: &Edits) -> Result<raw_pipeline::lut::LutMap, RenderError> {
        let mut map = raw_pipeline::lut::empty_luts();
        if let Some(id) = edits.referenced_lut_id() {
            let lut = self
                .luts
                .load(&id)
                .await
                .map_err(|e| RenderError::Lut(format!("{id}: {e}")))?;
            map.insert(id, lut);
        }
        Ok(map)
    }

    pub async fn dcp_for(
        &self,
        edits: &Edits,
        frame: &RawFrame,
    ) -> Result<Option<DcpSelection>, RenderError> {
        use raw_pipeline::edits::DcpMode;
        let dcp = &edits.color.dcp;
        if !frame.meta.is_raw || !dcp.is_active() {
            return Ok(None);
        }
        match dcp.mode {
            DcpMode::Off | DcpMode::Flat => Ok(None),
            DcpMode::Profile => match dcp.referenced_profile_id() {
                Some(id) => {
                    let profile = self
                        .dcp
                        .load(&id)
                        .await
                        .map_err(|e| RenderError::Dcp(format!("{id}: {e}")))?;
                    Ok(Some(DcpSelection { id, profile }))
                }
                None => Ok(None),
            },
            DcpMode::Auto => Ok(self
                .dcp
                .match_camera(&frame.meta.model)
                .await
                .map_err(|e| RenderError::Dcp(e.to_string()))?
                .map(|(id, profile)| DcpSelection { id, profile })),
        }
    }
}
