CREATE TABLE IF NOT EXISTS presets (
    id              TEXT NOT NULL,
    name            TEXT NOT NULL,
    group_name      TEXT,
    manifest_json   TEXT NOT NULL,
    schema_version  INTEGER NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE INDEX IF NOT EXISTS idx_presets_group_name ON presets(group_name, name);
