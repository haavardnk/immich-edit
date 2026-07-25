CREATE TABLE IF NOT EXISTS models (
    id            TEXT NOT NULL,
    catalog_id    TEXT,
    name          TEXT NOT NULL,
    kind          TEXT NOT NULL,
    content_hash  TEXT NOT NULL,
    size          INTEGER NOT NULL,
    license       TEXT NOT NULL,
    source_url    TEXT,
    deleted       INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_models_active_hash
    ON models(content_hash) WHERE deleted = 0;

CREATE UNIQUE INDEX IF NOT EXISTS idx_models_active_catalog
    ON models(catalog_id) WHERE deleted = 0 AND catalog_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_models_kind
    ON models(kind) WHERE deleted = 0;
