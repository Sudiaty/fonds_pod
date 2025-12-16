-- Add foreign key id columns to series, files, and items tables

-- Add fond_id to series table
ALTER TABLE series ADD COLUMN fond_id INTEGER DEFAULT 0;

-- Add series_id to files table
ALTER TABLE files ADD COLUMN series_id INTEGER DEFAULT 0;

-- Add file_id to items table
ALTER TABLE items ADD COLUMN file_id INTEGER DEFAULT 0;
