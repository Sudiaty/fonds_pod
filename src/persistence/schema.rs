use diesel::prelude::*;
use diesel::sql_query;
use std::error::Error;

/// Initialize the database schema by creating all necessary tables
pub fn init_schema(conn: &mut SqliteConnection) -> Result<(), Box<dyn Error>> {
    // Create fond_classifications table
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS fond_classifications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            parent_id INTEGER,
            active BOOLEAN NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_by TEXT NOT NULL,
            created_machine TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (parent_id) REFERENCES fond_classifications(id)
        )
        "#,
    )
    .execute(conn)?;

    // Create schemas table (id 自增主键)
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS schemas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            schema_no TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_by TEXT NOT NULL,
            created_machine TEXT NOT NULL,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(conn)?;

    // Create schema_items table (id 自增主键)
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            schema_id INTEGER NOT NULL,
            item_no TEXT NOT NULL,
            item_name TEXT NOT NULL,
            created_by TEXT NOT NULL,
            created_machine TEXT NOT NULL,
            created_at TEXT NOT NULL,
            UNIQUE (schema_id, item_no),
            FOREIGN KEY (schema_id) REFERENCES schemas(id)
        )
        "#,
    )
    .execute(conn)?;

    // Create fonds table
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS fonds (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fond_no TEXT NOT NULL UNIQUE,
            fond_classification_code TEXT NOT NULL DEFAULT '',
            name TEXT NOT NULL,
            created_by TEXT NOT NULL,
            created_machine TEXT NOT NULL,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(conn)?;

    // Create fond_schemas table
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS fond_schemas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fond_id INTEGER NOT NULL,
            schema_id INTEGER NOT NULL,
            schema_item_id INTEGER,  -- Nullable for dynamic schemas like Year
            sort_order INTEGER NOT NULL,
            created_by TEXT NOT NULL,
            created_machine TEXT NOT NULL,
            created_at TEXT NOT NULL,
            UNIQUE (fond_id, schema_id, schema_item_id),
            FOREIGN KEY (fond_id) REFERENCES fonds(id),
            FOREIGN KEY (schema_id) REFERENCES schemas(id),
            FOREIGN KEY (schema_item_id) REFERENCES schema_items(id)
        )
        "#,
    )
    .execute(conn)?;

    // Create series table
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS series (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fond_id INTEGER NOT NULL,
            series_no TEXT NOT NULL DEFAULT '',
            name TEXT NOT NULL,
            created_by TEXT NOT NULL,
            created_machine TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (fond_id) REFERENCES fonds(id)
        )
        "#,
    )
    .execute(conn)?;

    // Create files table
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            series_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            file_no TEXT NOT NULL DEFAULT '',
            path TEXT,
            created_by TEXT NOT NULL,
            created_machine TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (series_id) REFERENCES series(id)
        )
        "#,
    )
    .execute(conn)?;

    // Create items table
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL,
            item_no TEXT NOT NULL DEFAULT '',
            name TEXT NOT NULL,
            path TEXT,
            created_by TEXT NOT NULL,
            created_machine TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (file_id) REFERENCES files(id)
        )
        "#,
    )
    .execute(conn)?;

    // Create sequences table (drop and recreate to update schema)
    let _ = sql_query("DROP TABLE IF EXISTS sequences").execute(conn);
    sql_query(
        r#"
        CREATE TABLE sequences (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            prefix TEXT NOT NULL,
            next_value INTEGER NOT NULL DEFAULT 1,
            digits INTEGER NOT NULL DEFAULT 2,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(prefix)
        )
        "#,
    )
    .execute(conn)?;

    // Insert default Year schema if not exists
    sql_query(
        r#"
        INSERT OR IGNORE INTO schemas (schema_no, name, sort_order, created_by, created_machine, created_at)
        VALUES ('Year', 'Year', 0, 'system', 'system', CURRENT_TIMESTAMP)
        "#,
    )
    .execute(conn)?;

    // Add file_no and path columns to files table if they don't exist
    // Using PRAGMA table_info to check if columns exist before adding them
    let _ = sql_query("ALTER TABLE files ADD COLUMN file_no TEXT NOT NULL DEFAULT ''").execute(conn);
    let _ = sql_query("ALTER TABLE files ADD COLUMN path TEXT").execute(conn);

    // Add series_no and item_no columns if they don't exist
    let _ = sql_query("ALTER TABLE series ADD COLUMN series_no TEXT NOT NULL DEFAULT ''").execute(conn);
    let _ = sql_query("ALTER TABLE items ADD COLUMN item_no TEXT NOT NULL DEFAULT ''").execute(conn);

    Ok(())
}
