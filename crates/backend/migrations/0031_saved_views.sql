CREATE TABLE IF NOT EXISTS saved_views (
    id          TEXT NOT NULL PRIMARY KEY,
    user_id     TEXT NOT NULL,
    name        TEXT NOT NULL,
    url         TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_saved_views_owner ON saved_views (user_id, name);
