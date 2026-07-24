CREATE TABLE IF NOT EXISTS luts (
    id            TEXT NOT NULL,
    name          TEXT NOT NULL,
    content_hash  TEXT NOT NULL,
    size          INTEGER NOT NULL,
    lut_size      INTEGER NOT NULL,
    deleted       INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_luts_active_hash
    ON luts(content_hash) WHERE deleted = 0;
