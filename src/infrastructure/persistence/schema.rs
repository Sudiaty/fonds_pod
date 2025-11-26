// Database schema definition using Diesel ORM
// This file is automatically managed by Diesel migrations
// @generated automatically by Diesel CLI.

diesel::table! {
    schemas (schema_no) {
        schema_no -> Text,
        name -> Text,
    }
}

diesel::table! {
    schema_items (schema_no, item_no) {
        schema_no -> Text,
        item_no -> Text,
        item_name -> Text,
    }
}

diesel::table! {
    fond_classifications (code) {
        code -> Text,
        name -> Text,
        parent_id -> Nullable<Text>,
        is_active -> Bool,
    }
}

diesel::table! {
    fonds (fond_no) {
        fond_no -> Text,
        fond_classification_code -> Text,
        name -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    fond_schemas (fond_no, schema_no) {
        fond_no -> Text,
        schema_no -> Text,
        order_no -> Integer,
    }
}

diesel::table! {
    series (series_no) {
        series_no -> Text,
        fond_no -> Text,
        name -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    files (file_no) {
        file_no -> Text,
        series_no -> Text,
        name -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    items (item_no) {
        item_no -> Text,
        file_no -> Text,
        name -> Text,
        path -> Nullable<Text>,
        created_at -> Text,
    }
}

diesel::table! {
    sequences (prefix) {
        prefix -> Text,
        current_value -> Integer,
    }
}

diesel::joinable!(schema_items -> schemas (schema_no));
diesel::joinable!(fonds -> fond_classifications (fond_classification_code));
diesel::joinable!(fond_schemas -> fonds (fond_no));
diesel::joinable!(fond_schemas -> schemas (schema_no));
diesel::joinable!(series -> fonds (fond_no));
diesel::joinable!(files -> series (series_no));
diesel::joinable!(items -> files (file_no));

diesel::allow_tables_to_appear_in_same_query!(
    schemas,
    schema_items,
    fond_classifications,
    fonds,
    fond_schemas,
    series,
    files,
    items,
    sequences,
);

use diesel::sqlite::SqliteConnection;
use diesel::RunQueryDsl;

pub fn init_schema(connection: &mut SqliteConnection) -> Result<(), Box<dyn std::error::Error>> {
    // Create Schemas table
    diesel::sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS schemas (
            schema_no TEXT PRIMARY KEY,
            name TEXT NOT NULL
        )
        "#,
    )
    .execute(connection)?;

    // Create SchemaItems table
    diesel::sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_items (
            schema_no TEXT NOT NULL,
            item_no TEXT NOT NULL,
            item_name TEXT NOT NULL,
            PRIMARY KEY (schema_no, item_no),
            FOREIGN KEY (schema_no) REFERENCES schemas(schema_no)
        )
        "#,
    )
    .execute(connection)?;

    // Create FondClassifications table
    diesel::sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS fond_classifications (
            code TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            parent_id TEXT,
            is_active BOOLEAN NOT NULL DEFAULT 1,
            FOREIGN KEY (parent_id) REFERENCES fond_classifications(code)
        )
        "#,
    )
    .execute(connection)?;

    // Create Fonds table
    diesel::sql_query(
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
    .execute(connection)?;

    // Create FondSchemas table
    diesel::sql_query(
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
    .execute(connection)?;

    // Create Series table
    diesel::sql_query(
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
    .execute(connection)?;

    // Create Files table
    diesel::sql_query(
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
    .execute(connection)?;

    // Create Items table
    diesel::sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS items (
            item_no TEXT PRIMARY KEY,
            file_no TEXT NOT NULL,
            name TEXT NOT NULL,
            path TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (file_no) REFERENCES files(file_no)
        )
        "#,
    )
    .execute(connection)?;

    // Create Sequences table
    diesel::sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS sequences (
            prefix TEXT PRIMARY KEY,
            current_value INTEGER NOT NULL DEFAULT 0
        )
        "#,
    )
    .execute(connection)?;

    // Migration: Add path column to items table if it doesn't exist
    // Try to add the column - if it already exists, the error is ignored
    let _ = diesel::sql_query(
        "ALTER TABLE items ADD COLUMN path TEXT"
    )
    .execute(connection);

    // Initialize Year schema (special case - cannot be modified)
    diesel::sql_query(
        r#"
        INSERT OR IGNORE INTO schemas (schema_no, name) 
        VALUES ('Year', 'Year')
        "#,
    )
    .execute(connection)?;

    Ok(())
}
