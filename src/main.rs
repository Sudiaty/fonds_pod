// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
extern crate fonds_pod_lib;
mod app;

use std::error::Error;
use slint::ComponentHandle;
use fonds_pod_lib::services::{SettingsService, runtime_translations};
use fonds_pod_lib::AppWindow;

// 从环境变量读取版本号（由GitHub Actions注入）
pub const APP_VERSION: &str = env!("APP_VERSION");

fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志记录
    simple_logger::init_with_level(log::Level::Info)?;

    log::info!("Starting application version {}", APP_VERSION);

    // 0. 初始化gettext系统
    if let Err(e) = runtime_translations::init_gettext() {
        log::warn!("Failed to initialize gettext: {}", e);
    }

    // 1. 初始化应用服务
    let settings_service = SettingsService::new();
    let language = settings_service.get_language()?;

    // 2. 启动Slint应用程序
    let main_window = AppWindow::new()?;

    // 3. 设置语言 - 必须在创建 UI 之后设置
    if !language.is_empty() {
        // 使用 Slint 的 select_bundled_translation 动态选择翻译
        match slint::select_bundled_translation(&language) {
            Ok(_) => log::info!("Successfully selected bundled translation for: {}", language),
            Err(e) => log::warn!("Failed to select bundled translation for {}: {}", language, e),
        }
        
        if let Err(e) = runtime_translations::set_language(&language) {
            log::warn!("Failed to set language: {}, error: {}", language, e);
        } else {
            log::info!("Application language set to: {}", language);
        }
    }

    // 4. 初始化应用程序协调器
    let app_coordinator = app::App::initialize(&main_window);
    
    // 5. 设置所有 UI 回调
    app_coordinator.setup_ui_callbacks(&main_window);
    
    // 6. 运行主事件循环
    main_window.run()?;

    Ok(())
}
