CREATE TABLE IF NOT EXISTS user_preferences (
    user_id                TEXT NOT NULL PRIMARY KEY,
    click_target           TEXT NOT NULL,
    mask_overlay_color     TEXT,
    pinned_settings_json   TEXT,
    shortcuts_json         TEXT NOT NULL,
    desktop_sections_json  TEXT,
    touch_subviews_json    TEXT,
    updated_at             TEXT NOT NULL
);
