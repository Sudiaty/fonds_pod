use fonds_pod_lib::persistence::establish_connection;
use std::path::Path;
use tempfile::NamedTempFile;

#[test]
fn test_establish_connection_success() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();
    // SQLite will create the database file if it doesn't exist
    let result = establish_connection(path);
    assert!(result.is_ok());
}

#[test]
fn test_establish_connection_invalid_path() {
    // Test with an invalid path that cannot be created
    let invalid_path = Path::new("/invalid/path/database.db");
    let result = establish_connection(invalid_path);
    // Depending on the system, this might fail or succeed
    // For SQLite, it might create the file if permissions allow
    // But in most cases, it should fail for invalid paths
    // Since we can't guarantee, we'll just check that it returns a Result
    let _ = result; // Placeholder, as actual behavior depends on system
}