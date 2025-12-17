-- Modify fond_schemas table to allow NULL schema_item_id for dynamic schemas like Year
-- First drop the existing foreign key constraint
-- Note: SQLite doesn't support DROP CONSTRAINT directly, so we need to recreate the table

-- Create new fond_schemas table with nullable schema_item_id
CREATE TABLE fond_schemas_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fond_id INTEGER NOT NULL,
    schema_id INTEGER NOT NULL,
    schema_item_id INTEGER,  -- Now nullable
    sort_order INTEGER NOT NULL,
    created_by TEXT NOT NULL,
    created_machine TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE (fond_id, schema_id, schema_item_id),
    FOREIGN KEY (fond_id) REFERENCES fonds(id),
    FOREIGN KEY (schema_id) REFERENCES schemas(id),
    FOREIGN KEY (schema_item_id) REFERENCES schema_items(id)
);

-- Copy data from old table
INSERT INTO fond_schemas_new (id, fond_id, schema_id, schema_item_id, sort_order, created_by, created_machine, created_at)
SELECT id, fond_id, schema_id, schema_item_id, sort_order, created_by, created_machine, created_at
FROM fond_schemas;

-- Drop old table and rename new one
DROP TABLE fond_schemas;
ALTER TABLE fond_schemas_new RENAME TO fond_schemas;