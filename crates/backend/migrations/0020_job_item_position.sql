ALTER TABLE job_items ADD COLUMN position INTEGER NOT NULL DEFAULT 0;

UPDATE job_items
SET position = (
    SELECT COUNT(*) FROM job_items AS earlier
    WHERE earlier.job_id = job_items.job_id AND earlier.rowid <= job_items.rowid
);
