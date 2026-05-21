CREATE TABLE IF NOT EXISTS edits (
    asset_id          TEXT PRIMARY KEY NOT NULL,
    edits_json        TEXT NOT NULL,
    schema_version    INTEGER NOT NULL DEFAULT 1,
    renderer_version  TEXT NOT NULL,
    immich_updated_at TEXT,
    immich_checksum   TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_edits_updated_at ON edits(updated_at DESC);
