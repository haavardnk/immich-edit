CREATE TABLE IF NOT EXISTS raster_refs (
    user_id   TEXT NOT NULL,
    asset_id  TEXT NOT NULL,
    raster_id TEXT NOT NULL,
    PRIMARY KEY (user_id, asset_id, raster_id)
);

CREATE INDEX IF NOT EXISTS idx_raster_refs_raster ON raster_refs(raster_id);
