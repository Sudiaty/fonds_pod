-- Drop foreign key id columns from series, files, and items tables

-- Drop fond_id from series table
ALTER TABLE series DROP COLUMN fond_id;

-- Drop series_id from files table
ALTER TABLE files DROP COLUMN series_id;

-- Drop file_id from items table
ALTER TABLE items DROP COLUMN file_id;
