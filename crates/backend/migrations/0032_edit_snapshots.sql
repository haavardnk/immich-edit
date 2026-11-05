CREATE TABLE IF NOT EXISTS edit_snapshots (
    id             TEXT NOT NULL PRIMARY KEY,
    user_id        TEXT NOT NULL,
    asset_id       TEXT NOT NULL,
    name           TEXT NOT NULL,
    edits_json     TEXT NOT NULL,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_edit_snapshots_owner_asset_created
    ON edit_snapshots (user_id, asset_id, created_at);
