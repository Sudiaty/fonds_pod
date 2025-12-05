use fonds_pod_lib::persistence::{
    establish_connection,
    fond_classification_repository::FondClassificationsRepository,
};
use fonds_pod_lib::{models::fond_classification::FondClassification, GenericRepository, ActiveableRepository};
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
fn test_fond_classifications_repository() {
    let db_path = setup_test_db("fond_classification");
    let mut conn = establish_connection(Path::new(&db_path)).unwrap();
    let mut repo = FondClassificationsRepository::new(&mut conn);

    // Test create
    let classification = FondClassification {
        code: "GA".to_string(),
        name: "文化".to_string(),
        parent_id: None,
        active: true,
        ..Default::default()
    };

    let created_id = repo.create(classification).unwrap();
    assert!(created_id > 0);

    // Test find_by_id
    let found = repo.find_by_id(created_id).unwrap().unwrap();
    assert_eq!(found.code, "GA");
    assert_eq!(found.name, "文化");
    assert_eq!(found.parent_id, None);
    assert!(found.active);

    // Test find_active
    let active_classifications = repo.find_active().unwrap();
    assert_eq!(active_classifications.len(), 1);
    assert_eq!(active_classifications[0].code, "GA");

    // Test deactivate
    repo.deactivate(created_id).unwrap();

    // Test find_inactive
    let inactive_classifications = repo.find_inactive().unwrap();
    assert_eq!(inactive_classifications.len(), 1);
    assert_eq!(inactive_classifications[0].code, "GA");
    assert!(!inactive_classifications[0].active);

    // Test activate
    repo.activate(created_id).unwrap();
    let reactivated = repo.find_by_id(created_id).unwrap().unwrap();
    assert!(reactivated.active);
}