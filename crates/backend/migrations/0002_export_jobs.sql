CREATE TABLE IF NOT EXISTS export_jobs (
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
    PRIMARY KEY (asset_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_export_jobs_created_at ON export_jobs(created_at DESC);
