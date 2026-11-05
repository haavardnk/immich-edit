CREATE TABLE IF NOT EXISTS export_outputs (
    user_id          TEXT NOT NULL,
    asset_id         TEXT NOT NULL,
    preset           TEXT NOT NULL,
    immich_asset_id  TEXT NOT NULL,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_export_outputs_key
    ON export_outputs(user_id, asset_id, preset);

CREATE INDEX IF NOT EXISTS idx_export_outputs_immich
    ON export_outputs(user_id, immich_asset_id);

ALTER TABLE export_jobs ADD COLUMN replaced_asset_id TEXT;

ALTER TABLE export_jobs ADD COLUMN previous_asset_id TEXT;

ALTER TABLE export_jobs ADD COLUMN replace_asset_id TEXT;
