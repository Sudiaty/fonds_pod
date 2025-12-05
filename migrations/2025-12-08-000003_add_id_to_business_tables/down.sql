-- Rollback migration: Remove id, created_by, and created_machine columns

-- Rollback items table
CREATE TABLE items_old (
    item_no TEXT PRIMARY KEY,
    file_no TEXT NOT NULL,
    name TEXT NOT NULL,
    path TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (file_no) REFERENCES files(file_no)
);

INSERT INTO items_old (item_no, file_no, name, path, created_at)
SELECT item_no, file_no, name, path, created_at
FROM items;

DROP TABLE items;
ALTER TABLE items_old RENAME TO items;

-- Rollback files table
CREATE TABLE files_old (
    file_no TEXT PRIMARY KEY,
    series_no TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (series_no) REFERENCES series(series_no)
);

INSERT INTO files_old (file_no, series_no, name, created_at)
SELECT file_no, series_no, name, created_at
FROM files;

DROP TABLE files;
ALTER TABLE files_old RENAME TO files;

-- Rollback series table
CREATE TABLE series_old (
    series_no TEXT PRIMARY KEY,
    fond_no TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (fond_no) REFERENCES fonds(fond_no)
);

INSERT INTO series_old (series_no, fond_no, name, created_at)
SELECT series_no, fond_no, name, created_at
FROM series;

DROP TABLE series;
ALTER TABLE series_old RENAME TO series;

-- Rollback fond_schemas table
CREATE TABLE fond_schemas_old (
    fond_no TEXT NOT NULL,
    schema_no TEXT NOT NULL,
    order_no INTEGER NOT NULL,
    PRIMARY KEY (fond_no, schema_no),
    FOREIGN KEY (fond_no) REFERENCES fonds(fond_no),
    FOREIGN KEY (schema_no) REFERENCES schemas(schema_no)
);

INSERT INTO fond_schemas_old (fond_no, schema_no, order_no)
SELECT fond_no, schema_no, order_no
FROM fond_schemas;

DROP TABLE fond_schemas;
ALTER TABLE fond_schemas_old RENAME TO fond_schemas;

-- Rollback fonds table
CREATE TABLE fonds_old (
    fond_no TEXT PRIMARY KEY,
    fond_classification_code TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (fond_classification_code) REFERENCES fond_classifications(code)
);

INSERT INTO fonds_old (fond_no, fond_classification_code, name, created_at)
SELECT fond_no, fond_classification_code, name, created_at
FROM fonds;

DROP TABLE fonds;
ALTER TABLE fonds_old RENAME TO fonds;
