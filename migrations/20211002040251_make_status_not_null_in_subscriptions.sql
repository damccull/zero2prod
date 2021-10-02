-- Wrap entire migration in a transaction to make sure it succeeds or fails as a unit
BEGIN;
    -- Backfill 'status' for historical entries
    UPDATE subscriptions
        SET status = 'confirmed'
        WHERE status IS NULL;
    -- Make 'status' mandatory
    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;
