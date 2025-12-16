-- Add file_no and path fields to files table
ALTER TABLE files ADD COLUMN file_no TEXT;
ALTER TABLE files ADD COLUMN path TEXT;