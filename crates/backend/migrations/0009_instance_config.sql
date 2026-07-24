CREATE TABLE instance_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    server_epoch INTEGER NOT NULL DEFAULT 0,
    immich_url TEXT,
    configured_at TEXT,
    key_version INTEGER NOT NULL DEFAULT 1
);

INSERT INTO instance_config (id, server_epoch) VALUES (1, 0);
