ALTER TABLE issue_delivery_queue
ADD COLUMN retries INTEGER NOT NULL DEFAULT 0,
ADD COLUMN retry_after timestamptz NOT NULL;
