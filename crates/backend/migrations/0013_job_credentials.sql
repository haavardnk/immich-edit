CREATE TABLE IF NOT EXISTS job_credentials (
    job_id       TEXT PRIMARY KEY,
    ciphertext   BLOB NOT NULL,
    nonce        BLOB NOT NULL,
    key_version  INTEGER NOT NULL,
    auth_kind    TEXT NOT NULL,
    FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE
);
