ALTER TABLE fond_schemas ADD COLUMN fond_id INTEGER;
ALTER TABLE fond_schemas ADD COLUMN schema_id INTEGER;
ALTER TABLE fond_schemas ADD COLUMN schema_item_id INTEGER;
ALTER TABLE fond_schemas DROP COLUMN fond_no;
ALTER TABLE fond_schemas DROP COLUMN schema_no;