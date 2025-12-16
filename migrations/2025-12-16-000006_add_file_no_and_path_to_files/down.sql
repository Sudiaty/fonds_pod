-- Remove file_no and path fields from files table
ALTER TABLE files DROP COLUMN file_no;
ALTER TABLE files DROP COLUMN path;