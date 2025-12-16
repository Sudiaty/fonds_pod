-- Add series_no and item_no fields
ALTER TABLE series ADD COLUMN series_no TEXT;
ALTER TABLE items ADD COLUMN item_no TEXT;