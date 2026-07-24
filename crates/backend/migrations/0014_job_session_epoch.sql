ALTER TABLE jobs ADD COLUMN server_epoch INTEGER NOT NULL DEFAULT 0;
ALTER TABLE jobs ADD COLUMN auth_session_id TEXT;

CREATE INDEX idx_jobs_session ON jobs (auth_session_id, status);
