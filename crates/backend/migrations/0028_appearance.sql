CREATE TABLE IF NOT EXISTS user_appearance (
    user_id       TEXT NOT NULL PRIMARY KEY,
    mode          TEXT NOT NULL,
    light_theme   TEXT NOT NULL,
    dark_theme    TEXT NOT NULL,
    accents_json  TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);
