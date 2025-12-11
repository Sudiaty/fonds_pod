use fonds_pod_lib::viewmodels::FondViewModel;
use fonds_pod_lib::persistence::{FondsRepository, establish_connection};
use std::rc::Rc;
use std::cell::RefCell;
use slint::Model;

#[test]
fn test_fond_vm_add() {
    // 使用内存数据库
    let db_path = std::path::PathBuf::from(":memory:");
    let conn = establish_connection(&db_path).expect("Failed to establish connection");
    
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
