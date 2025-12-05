-- Add id column to fond_classifications table
ALTER TABLE fond_classifications ADD COLUMN id INTEGER PRIMARY KEY AUTOINCREMENT;
ALTER TABLE fond_classifications ADD COLUMN created_by TEXT NOT NULL DEFAULT '';
ALTER TABLE fond_classifications ADD COLUMN created_machine TEXT NOT NULL DEFAULT '';
ALTER TABLE fond_classifications ADD COLUMN created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP;