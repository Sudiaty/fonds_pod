use diesel::prelude::*;
use diesel::sql_query;
use std::error::Error;

/// Initialize the database schema by creating all necessary tables
pub fn init_schema(conn: &mut SqliteConnection) -> Result<(), Box<dyn Error>> {
    // Create schemas table (id 自增主键)
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS schemas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            schema_no TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
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

    // Create fond_classifications table
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS fond_classifications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            parent_id INTEGER,
            active BOOLEAN NOT NULL DEFAULT 1,
            created_by TEXT NOT NULL,
            created_machine TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (parent_id) REFERENCES fond_classifications(id)
        )
        "#,
    )
    .execute(conn)?;

    // Create fonds table
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS fonds (
            fond_no TEXT PRIMARY KEY,
            fond_classification_code TEXT NOT NULL,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (fond_classification_code) REFERENCES fond_classifications(code)
        )
        "#,
    )
    .execute(conn)?;

    // Create fond_schemas table
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS fond_schemas (
            fond_no TEXT NOT NULL,
            schema_no TEXT NOT NULL,
            order_no INTEGER NOT NULL,
            PRIMARY KEY (fond_no, schema_no),
            FOREIGN KEY (fond_no) REFERENCES fonds(fond_no),
            FOREIGN KEY (schema_no) REFERENCES schemas(schema_no)
        )
        "#,
    )
    .execute(conn)?;

    // Create series table
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS series (
            series_no TEXT PRIMARY KEY,
            fond_no TEXT NOT NULL,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (fond_no) REFERENCES fonds(fond_no)
        )
        "#,
    )
    .execute(conn)?;

    // Create files table
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS files (
            file_no TEXT PRIMARY KEY,
            series_no TEXT NOT NULL,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (series_no) REFERENCES series(series_no)
        )
        "#,
    )
    .execute(conn)?;

    // Create items table
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS items (
            item_no TEXT PRIMARY KEY,
            file_no TEXT NOT NULL,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (file_no) REFERENCES files(file_no)
        )
        "#,
    )
    .execute(conn)?;

    // Create sequences table
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS sequences (
            prefix TEXT PRIMARY KEY,
            current_value INTEGER NOT NULL DEFAULT 0
        )
        "#,
    )
    .execute(conn)?;

    Ok(())
}
