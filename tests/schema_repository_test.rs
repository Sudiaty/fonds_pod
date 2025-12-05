use fonds_pod_lib::persistence::{
    establish_connection, 
    schema_repository::SchemaRepository,
    GenericRepository,
};
use fonds_pod_lib::models::schema::Schema;
use std::path::Path;
use std::fs;


fn setup_test_db(test_name: &str) -> String {
    let path = format!(".fondspod_test_{}.db", test_name);
    if Path::new(&path).exists() {
        let _ = fs::remove_file(&path);
    }
    path
}

#[test]
fn test_schema_repository_insert_and_find() {
    let path = setup_test_db("insert_and_find");
    let mut conn = establish_connection(Path::new(&path)).unwrap();
    let mut repo = SchemaRepository::new(&mut conn);

    // 使用 create 方法，自动设置 created_at，返回自增 id
    let new_id = repo.create(Schema { schema_no: "S001".into(), name: "Test Schema".into(), ..Default::default() }).unwrap();
    assert!(new_id > 0);

    // Find by id
    let found = repo.find_by_id(new_id).unwrap();
    assert!(found.is_some());
    let found_schema = found.unwrap();
    assert_eq!(found_schema.id, new_id);
    assert_eq!(found_schema.schema_no, "S001");
    assert_eq!(found_schema.name, "Test Schema");
    // 验证 created_at 已被自动设置
    assert!(found_schema.created_at > chrono::NaiveDateTime::default());
    
    // 验证 created_by 和 created_machine 已被自动设置
    assert!(!found_schema.created_by.is_empty());
    assert!(!found_schema.created_machine.is_empty());
    println!("Created by: {}", found_schema.created_by);
    println!("Created machine: {}", found_schema.created_machine);

    // Find non-existent
    let not_found = repo.find_by_id(9999).unwrap();
    assert!(not_found.is_none());
}

#[test]
fn test_schema_repository_find_all() {
    let path = setup_test_db("find_all");
    let mut conn = establish_connection(Path::new(&path)).unwrap();
    let mut repo = SchemaRepository::new(&mut conn);

    repo.create(Schema { schema_no: "S001".into(), name: "Schema 1".into(), ..Default::default() }).unwrap();
    repo.create(Schema { schema_no: "S002".into(), name: "Schema 2".into(), ..Default::default() }).unwrap();

    let all = repo.find_all().unwrap();
    assert_eq!(all.len(), 2);
    assert!(all.iter().any(|s| s.schema_no == "S001"));
    assert!(all.iter().any(|s| s.schema_no == "S002"));
}

#[test]
fn test_schema_repository_update() {
    let path = setup_test_db("update");
    let mut conn = establish_connection(Path::new(&path)).unwrap();
    let mut repo = SchemaRepository::new(&mut conn);

    let new_id = repo.create(Schema { schema_no: "S001".into(), name: "Original Name".into(), ..Default::default() }).unwrap();

    // 获取已创建的记录并更新
    let mut schema = repo.find_by_id(new_id).unwrap().unwrap();
    schema.name = "Updated Name".to_string();
    repo.update(&schema).unwrap();

    // Verify update
    let found = repo.find_by_id(new_id).unwrap().unwrap();
    assert_eq!(found.name, "Updated Name");
}

#[test]
fn test_schema_repository_delete() {
    let path = setup_test_db("delete");
    let mut conn = establish_connection(Path::new(&path)).unwrap();
    let mut repo = SchemaRepository::new(&mut conn);

    let new_id = repo.create(Schema { schema_no: "S001".into(), name: "Test Schema".into(), ..Default::default() }).unwrap();

    // Verify exists
    let found = repo.find_by_id(new_id).unwrap();
    assert!(found.is_some());

    // Delete
    repo.delete(new_id).unwrap();

    // Verify deleted
    let found_after = repo.find_by_id(new_id).unwrap();
    assert!(found_after.is_none());
}