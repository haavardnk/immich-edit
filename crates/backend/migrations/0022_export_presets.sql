CREATE TABLE IF NOT EXISTS export_presets (
    id          TEXT NOT NULL PRIMARY KEY,
    user_id     TEXT NOT NULL,
    name        TEXT NOT NULL,
    form_json   TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_export_presets_owner ON export_presets (user_id, name);
