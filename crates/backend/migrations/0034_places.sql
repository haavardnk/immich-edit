CREATE TABLE IF NOT EXISTS saved_places (
    id          TEXT NOT NULL PRIMARY KEY,
    user_id     TEXT NOT NULL,
    name        TEXT NOT NULL,
    lat         REAL NOT NULL,
    lon         REAL NOT NULL,
    city        TEXT,
    state       TEXT,
    country     TEXT,
    created_at  TEXT NOT NULL,
    used_at     TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_saved_places_name
    ON saved_places (user_id, name COLLATE NOCASE);

CREATE TABLE IF NOT EXISTS recent_places (
    id          TEXT NOT NULL PRIMARY KEY,
    user_id     TEXT NOT NULL,
    lat         REAL NOT NULL,
    lon         REAL NOT NULL,
    city        TEXT,
    state       TEXT,
    country     TEXT,
    used_at     TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_recent_places_owner ON recent_places (user_id, used_at);
