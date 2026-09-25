CREATE TABLE IF NOT EXISTS watermarks (
    id            TEXT NOT NULL,
    name          TEXT NOT NULL,
    content_hash  TEXT NOT NULL,
    size          INTEGER NOT NULL,
    width         INTEGER NOT NULL,
    height        INTEGER NOT NULL,
    deleted       INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_watermarks_active_hash
    ON watermarks(content_hash) WHERE deleted = 0;
