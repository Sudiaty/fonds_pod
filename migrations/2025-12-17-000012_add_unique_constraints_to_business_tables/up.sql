-- Add unique constraints to prevent duplicate numbers within the same parent
-- Series numbers must be unique within the same fond
CREATE UNIQUE INDEX IF NOT EXISTS idx_series_fond_series_no ON series(fond_id, series_no);

-- File numbers must be unique within the same series
CREATE UNIQUE INDEX IF NOT EXISTS idx_files_series_file_no ON files(series_id, file_no);

-- Item numbers must be unique within the same file
CREATE UNIQUE INDEX IF NOT EXISTS idx_items_file_item_no ON items(file_id, item_no);