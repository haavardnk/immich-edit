CREATE TABLE IF NOT EXISTS dcp_profiles (
    id            TEXT NOT NULL,
    name          TEXT NOT NULL,
    camera_model  TEXT,
    copyright     TEXT,
    content_hash  TEXT NOT NULL,
    size          INTEGER NOT NULL,
    bundled       INTEGER NOT NULL DEFAULT 0,
    deleted       INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_dcp_active_hash
    ON dcp_profiles(content_hash) WHERE deleted = 0;

CREATE INDEX IF NOT EXISTS idx_dcp_camera
    ON dcp_profiles(camera_model) WHERE deleted = 0;
