CREATE TABLE edits_new (
    user_id           TEXT NOT NULL,
    asset_id          TEXT NOT NULL,
    edits_json        TEXT NOT NULL,
    schema_version    INTEGER NOT NULL DEFAULT 1,
    renderer_version  TEXT NOT NULL,
    immich_updated_at TEXT,
    immich_checksum   TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL,
    PRIMARY KEY (user_id, asset_id)
);

INSERT INTO edits_new (user_id, asset_id, edits_json, schema_version, renderer_version, immich_updated_at, immich_checksum, created_at, updated_at)
    SELECT '00000000-0000-0000-0000-000000000000', asset_id, edits_json, schema_version, renderer_version, immich_updated_at, immich_checksum, created_at, updated_at FROM edits;

DROP TABLE edits;
ALTER TABLE edits_new RENAME TO edits;
CREATE INDEX idx_edits_updated_at ON edits (updated_at DESC);
CREATE INDEX idx_edits_owner ON edits (user_id, updated_at DESC);

ALTER TABLE edits_history ADD COLUMN user_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX idx_edits_history_owner ON edits_history (user_id, asset_id, created_at DESC);

CREATE TABLE export_jobs_new (
    user_id          TEXT NOT NULL,
    asset_id         TEXT NOT NULL,
    idempotency_key  TEXT NOT NULL,
    request_hash     TEXT NOT NULL,
    status           TEXT NOT NULL,
    immich_asset_id  TEXT,
    filename         TEXT,
    upload_status    TEXT,
    warnings_json    TEXT NOT NULL DEFAULT '[]',
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL,
    PRIMARY KEY (user_id, asset_id, idempotency_key)
);

INSERT INTO export_jobs_new (user_id, asset_id, idempotency_key, request_hash, status, immich_asset_id, filename, upload_status, warnings_json, created_at, updated_at)
    SELECT '00000000-0000-0000-0000-000000000000', asset_id, idempotency_key, request_hash, status, immich_asset_id, filename, upload_status, warnings_json, created_at, updated_at FROM export_jobs;

DROP TABLE export_jobs;
ALTER TABLE export_jobs_new RENAME TO export_jobs;
CREATE INDEX idx_export_jobs_created_at ON export_jobs (created_at DESC);

ALTER TABLE presets ADD COLUMN user_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX idx_presets_owner ON presets (user_id, group_name, name);

ALTER TABLE jobs ADD COLUMN user_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX idx_jobs_owner ON jobs (user_id, status, created_at DESC);
