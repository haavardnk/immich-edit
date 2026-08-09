CREATE TABLE IF NOT EXISTS asset_copies (
    user_id         TEXT NOT NULL,
    id              TEXT NOT NULL,
    source_asset_id TEXT NOT NULL,
    idx             INTEGER NOT NULL,
    name            TEXT,
    deleted         INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL,
    PRIMARY KEY (user_id, id)
);

CREATE INDEX IF NOT EXISTS idx_asset_copies_source
    ON asset_copies(user_id, source_asset_id, idx);
