-- Remove no fields from series, files, and items tables
-- These are being replaced by id-based foreign keys

-- Remove series_no from series table
ALTER TABLE series DROP COLUMN series_no;

-- Remove file_no from files table
ALTER TABLE files DROP COLUMN file_no;

-- Remove item_no from items table
ALTER TABLE items DROP COLUMN item_no;
