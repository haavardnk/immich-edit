CREATE TABLE IF NOT EXISTS user_preferences (
    user_id                TEXT NOT NULL PRIMARY KEY,
    mask_overlay_color     TEXT,
    pinned_settings_json   TEXT,
    shortcuts_json         TEXT NOT NULL,
    desktop_sections_json  TEXT,
    touch_subviews_json    TEXT,
    crop_ratios_json       TEXT,
    updated_at             TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS saved_views (
    id          TEXT NOT NULL PRIMARY KEY,
    user_id     TEXT NOT NULL,
    name        TEXT NOT NULL,
    url         TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_saved_views_owner ON saved_views (user_id, name);

ALTER TABLE presets ADD COLUMN favorite INTEGER NOT NULL DEFAULT 0;
