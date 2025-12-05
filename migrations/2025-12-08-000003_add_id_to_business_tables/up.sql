-- Add id, created_by, and created_machine columns to fonds table
-- This migration adds the required fields for Creatable trait

-- Step 1: Create new fonds table with id
CREATE TABLE fonds_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fond_no TEXT NOT NULL UNIQUE,
    fond_classification_code TEXT NOT NULL,
    name TEXT NOT NULL,
    created_by TEXT NOT NULL,
    created_machine TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (fond_classification_code) REFERENCES fond_classifications(code)
);

-- Step 2: Migrate data from old table (use default values for new fields)
INSERT INTO fonds_new (fond_no, fond_classification_code, name, created_by, created_machine, created_at)
SELECT fond_no, fond_classification_code, name, 'migration', 'migration', created_at
FROM fonds;

-- Step 3: Drop old table and rename new table
DROP TABLE fonds;
ALTER TABLE fonds_new RENAME TO fonds;

-- Add id, created_by, and created_machine columns to fond_schemas table
CREATE TABLE fond_schemas_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fond_no TEXT NOT NULL,
    schema_no TEXT NOT NULL,
    order_no INTEGER NOT NULL,
    created_by TEXT NOT NULL,
    created_machine TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE (fond_no, schema_no),
    FOREIGN KEY (fond_no) REFERENCES fonds(fond_no),
    FOREIGN KEY (schema_no) REFERENCES schemas(schema_no)
);

INSERT INTO fond_schemas_new (fond_no, schema_no, order_no, created_by, created_machine, created_at)
SELECT fond_no, schema_no, order_no, 'migration', 'migration', datetime('now')
FROM fond_schemas;

DROP TABLE fond_schemas;
ALTER TABLE fond_schemas_new RENAME TO fond_schemas;

-- Add id, created_by, and created_machine columns to series table
CREATE TABLE series_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    series_no TEXT NOT NULL UNIQUE,
    fond_no TEXT NOT NULL,
    name TEXT NOT NULL,
    created_by TEXT NOT NULL,
    created_machine TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (fond_no) REFERENCES fonds(fond_no)
);

INSERT INTO series_new (series_no, fond_no, name, created_by, created_machine, created_at)
SELECT series_no, fond_no, name, 'migration', 'migration', created_at
FROM series;

DROP TABLE series;
ALTER TABLE series_new RENAME TO series;

-- Add id, created_by, and created_machine columns to files table
CREATE TABLE files_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_no TEXT NOT NULL UNIQUE,
    series_no TEXT NOT NULL,
    name TEXT NOT NULL,
    created_by TEXT NOT NULL,
    created_machine TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (series_no) REFERENCES series(series_no)
);

INSERT INTO files_new (file_no, series_no, name, created_by, created_machine, created_at)
SELECT file_no, series_no, name, 'migration', 'migration', created_at
FROM files;

DROP TABLE files;
ALTER TABLE files_new RENAME TO files;

-- Add id, created_by, and created_machine columns to items table
CREATE TABLE items_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_no TEXT NOT NULL UNIQUE,
    file_no TEXT NOT NULL,
    name TEXT NOT NULL,
    path TEXT,
    created_by TEXT NOT NULL,
    created_machine TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (file_no) REFERENCES files(file_no)
);

INSERT INTO items_new (item_no, file_no, name, path, created_by, created_machine, created_at)
SELECT item_no, file_no, name, path, 'migration', 'migration', created_at
FROM items;

DROP TABLE items;
ALTER TABLE items_new RENAME TO items;
