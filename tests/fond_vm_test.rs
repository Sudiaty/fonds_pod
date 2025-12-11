use fonds_pod_lib::viewmodels::FondViewModel;
use fonds_pod_lib::persistence::establish_connection;
use fonds_pod_lib::persistence::schema;
use fonds_pod_lib::persistence::FondsRepository;
use fonds_pod_lib::CrudViewModelBase;
use std::rc::Rc;
use std::cell::RefCell;
use slint::Model;
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
fn test_fond_vm_add() {
    let db_path = setup_test_db("fond_vm");
    let conn = establish_connection(Path::new(&db_path)).expect("Failed to establish connection");
    
    // Initialize schema
    schema::init_schema(&mut *conn.borrow_mut()).expect("Failed to initialize schema");
    
    let repo = Rc::new(RefCell::new(FondsRepository::new(conn)));
    let vm = FondViewModel::new(repo);
    
    // 初始应该是空的
    vm.load();
    assert_eq!(vm.get_items().row_count(), 0);
    
    // 添加一个fond
    vm.add();
    assert_eq!(vm.get_items().row_count(), 1);
    
    // 再添加一个
    vm.add();
    assert_eq!(vm.get_items().row_count(), 2);
    
    println!("Test passed: FondViewModel add functionality works!");
}
