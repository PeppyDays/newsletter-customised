BEGIN;
UPDATE subscribers SET status = 'Confirmed' WHERE status IS NULL;
ALTER TABLE subscribers ALTER COLUMN status SET NOT NULL;
COMMIT;
