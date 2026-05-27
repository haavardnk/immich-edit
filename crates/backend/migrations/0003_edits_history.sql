CREATE TABLE IF NOT EXISTS edits_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id TEXT NOT NULL,
    manifest_hash TEXT NOT NULL,
    edits_json TEXT,
    deleted INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_edits_history_asset_created
    ON edits_history (asset_id, created_at DESC);
