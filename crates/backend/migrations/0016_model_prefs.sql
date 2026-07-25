CREATE TABLE IF NOT EXISTS model_prefs (
    kind       TEXT NOT NULL,
    catalog_id TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (kind)
);
