-- Add sort_order column to fond_classifications table
ALTER TABLE fond_classifications ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;