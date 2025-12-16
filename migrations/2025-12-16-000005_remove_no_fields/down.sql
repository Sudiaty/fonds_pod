-- Add back the no fields if migration is rolled back
-- Note: This is a best-effort rollback; original values may not be recoverable

-- Add series_no back to series table
ALTER TABLE series ADD COLUMN series_no TEXT DEFAULT '';

-- Add file_no back to files table
ALTER TABLE files ADD COLUMN file_no TEXT DEFAULT '';

-- Add item_no back to items table
ALTER TABLE items ADD COLUMN item_no TEXT DEFAULT '';
