// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
extern crate fonds_pod_lib;
mod app;

use std::error::Error;

// 从环境变量读取版本号（由GitHub Actions注入）
pub const APP_VERSION: &str = env!("APP_VERSION");

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志记录
    simple_logger::init_with_level(log::Level::Info)?;

    log::info!("Starting application version {}", APP_VERSION);

    // 1. 启动Slint应用程序
    let main_window = MainWindow::new()?;

    // 2. 初始化应用程序协调器
    let app_coordinator = app::App::initialize(&main_window);
    
    // 3. 设置所有 UI 回调
    app_coordinator.setup_ui_callbacks(&main_window);
    
    // 4. 运行主事件循环
    main_window.run()?;

    Ok(())
}
