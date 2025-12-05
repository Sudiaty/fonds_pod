-- Remove added columns from fond_classifications table
ALTER TABLE fond_classifications DROP COLUMN id;
ALTER TABLE fond_classifications DROP COLUMN created_by;
ALTER TABLE fond_classifications DROP COLUMN created_machine;
ALTER TABLE fond_classifications DROP COLUMN created_at;