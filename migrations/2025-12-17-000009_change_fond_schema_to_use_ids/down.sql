ALTER TABLE fond_schemas ADD COLUMN fond_no TEXT;
ALTER TABLE fond_schemas ADD COLUMN schema_no TEXT;
ALTER TABLE fond_schemas DROP COLUMN fond_id;
ALTER TABLE fond_schemas DROP COLUMN schema_id;
ALTER TABLE fond_schemas DROP COLUMN schema_item_id;