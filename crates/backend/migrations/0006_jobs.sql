CREATE TABLE IF NOT EXISTS jobs (
    id            TEXT PRIMARY KEY,
    kind          TEXT NOT NULL,
    status        TEXT NOT NULL,
    target_json   TEXT NOT NULL DEFAULT 'null',
    params_json   TEXT NOT NULL DEFAULT 'null',
    total         INTEGER NOT NULL DEFAULT 0,
    completed     INTEGER NOT NULL DEFAULT 0,
    failed        INTEGER NOT NULL DEFAULT 0,
    cancelled_at  TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status, created_at DESC);

CREATE TABLE IF NOT EXISTS job_items (
    id               TEXT PRIMARY KEY,
    job_id           TEXT NOT NULL,
    asset_id         TEXT NOT NULL,
    status           TEXT NOT NULL,
    error            TEXT,
    result_json      TEXT,
    idempotency_key  TEXT,
    attempts         INTEGER NOT NULL DEFAULT 0,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL,
    FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_job_items_job_status ON job_items(job_id, status);
CREATE INDEX IF NOT EXISTS idx_job_items_status ON job_items(status);
