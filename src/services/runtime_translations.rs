/// Runtime translation loader using PO files
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static TRANSLATIONS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
static CURRENT_LANGUAGE: OnceLock<Mutex<String>> = OnceLock::new();

/// Initialize translation system
pub fn init_gettext() -> Result<(), Box<dyn Error>> {
    TRANSLATIONS.set(Mutex::new(HashMap::new()))
        .map_err(|_| "Failed to initialize translations")?;
    CURRENT_LANGUAGE.set(Mutex::new("zh_CN".to_string()))
        .map_err(|_| "Failed to initialize current language")?;
    
    log::info!("Translation system initialized");
    Ok(())
}

/// Load translations for a specific language from PO file
pub fn set_language(language: &str) -> Result<(), Box<dyn Error>> {
    let po_path = format!("ui/locale/{}/LC_MESSAGES/fonds-pod.po", language);
    let po_file_path = Path::new(&po_path);
    
    if po_file_path.exists() {
        // Parse PO file manually
        match fs::read_to_string(po_file_path) {
            Ok(po_content) => {
                let translations = TRANSLATIONS.get().unwrap();
                let mut trans_map = translations.lock().unwrap();
                trans_map.clear();
                
                // Simple PO file parser
                let mut msgid = String::new();
                let mut msgstr = String::new();
                let mut in_msgid = false;
                let mut in_msgstr = false;
                
                for line in po_content.lines() {
                    let line = line.trim();
                    if line.starts_with("msgid ") {
                        if !msgid.is_empty() && !msgstr.is_empty() {
                            trans_map.insert(msgid.clone(), msgstr.clone());
                        }
                        msgid = line[6..].trim_matches('"').to_string();
                        msgstr.clear();
                        in_msgid = true;
                        in_msgstr = false;
                    } else if line.starts_with("msgstr ") {
                        msgstr = line[7..].trim_matches('"').to_string();
                        in_msgid = false;
                        in_msgstr = true;
                    } else if line.starts_with('"') && line.ends_with('"') {
                        let content = line.trim_matches('"');
                        if in_msgid {
                            msgid.push_str(content);
                        } else if in_msgstr {
                            msgstr.push_str(content);
                        }
                    }
                }
                
                // Add last entry if any
                if !msgid.is_empty() && !msgstr.is_empty() {
                    trans_map.insert(msgid, msgstr);
                }
                
                // Update current language
                if let Some(current_lang) = CURRENT_LANGUAGE.get() {
                    *current_lang.lock().unwrap() = language.to_string();
                }
                
                log::info!("Loaded {} translations for language: {}", trans_map.len(), language);
            }
            Err(e) => {
                log::warn!("Failed to read PO file {}: {}", po_path, e);
                // Fall back to hardcoded translations
                load_fallback_translations(language)?;
            }
        }
    } else {
        log::warn!("PO file not found: {}, using fallback", po_path);
        load_fallback_translations(language)?;
    }
    
    Ok(())
}

/// Load fallback translations when PO file is not available
fn load_fallback_translations(language: &str) -> Result<(), Box<dyn Error>> {
    let translations = TRANSLATIONS.get().unwrap();
    let mut trans_map = translations.lock().unwrap();
    trans_map.clear();
    
    let fallback_translations = match language {
        "zh_CN" => get_chinese_translations(),
        "en_US" => get_english_translations(),
        _ => get_chinese_translations(),
    };
    
    for (key, value) in fallback_translations {
        trans_map.insert(key, value);
    }
    
    log::info!("Loaded {} fallback translations for language: {}", trans_map.len(), language);
    Ok(())
}

/// Get a translation for a message ID
pub fn gettext_tr(msgid: &str) -> String {
    if let Some(translations) = TRANSLATIONS.get() {
        if let Ok(trans_map) = translations.lock() {
            if let Some(translation) = trans_map.get(msgid) {
                return translation.clone();
            }
        }
    }
    
    // Return the key if no translation found
    msgid.to_string()
}

fn get_chinese_translations() -> Vec<(String, String)> {
    [
        // Navigation
        ("nav_fonds", "全宗管理"),
        ("nav_schema", "方案管理"), 
        ("nav_classification", "分类管理"),
        ("nav_settings", "系统设置"),
        ("nav_about", "关于"),
        
        // About page
        ("about_title", "FondsPod 档案管理系统"),
        ("about_description", "FondsPod 是一个现代化的档案管理系统，采用 Rust 和 Slint 构建。它提供了完整的档案分类、方案管理和全宗管理功能，帮助您高效地组织和管理档案资料。"),
        ("about_version", "版本:"),
        ("about_author", "作者:"),
        ("about_license", "许可证:"),
        ("about_website", "项目地址:"),
        ("check_update", "检查更新"),
        
        // Common UI messages
        ("language_applied", "语言已应用"),
        ("code_exists", "代码已存在"),
        ("create_failed", "创建失败"),
        ("tray_show_window", "显示窗口"),
        ("tray_quit", "退出"),
        ("delete_failed", "删除失败"),
        ("cannot_delete", "无法删除"),
        ("activate_failed", "激活失败"),
        ("deactivate_failed", "停用失败"),
        ("export_failed", "导出失败"),
        ("import_failed", "导入失败"),
        ("no_archive_selected", "未选择档案库"),
        ("import_success", "导入成功"),
        ("export_success", "导出成功"),
        ("read_file_failed", "读取文件失败"),
        
        // Settings page
        ("label_language", "语言"),
        ("label_archive_libraries", "档案库"),
        ("label_name", "名称"),
        ("label_path", "路径"),
        ("btn_cancel", "取消"),
        ("btn_apply", "应用"),
        ("dialog_add_archive_title", "添加档案库"),
        ("dialog_rename_archive_title", "重命名档案库"),
    ].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

fn get_english_translations() -> Vec<(String, String)> {
    [
        // Navigation
        ("nav_fonds", "Fonds Management"),
        ("nav_schema", "Schema Management"),
        ("nav_classification", "Classification Management"),
        ("nav_settings", "Settings"),
        ("nav_about", "About"),
        
        // About page
        ("about_title", "FondsPod Archive Management System"),
        ("about_description", "FondsPod is a modern archive management system built with Rust and Slint. It provides comprehensive archive classification, schema management, and fonds management features to help you efficiently organize and manage archival materials."),
        ("about_version", "Version:"),
        ("about_author", "Author:"),
        ("about_license", "License:"),
        ("about_website", "Website:"),
        ("check_update", "Check Update"),
        
        // Common UI messages
        ("language_applied", "Language Applied"),
        ("code_exists", "Code Already Exists"),
        ("create_failed", "Create Failed"),
        ("tray_show_window", "Show Window"),
        ("tray_quit", "Quit"),
        ("delete_failed", "Delete Failed"),
        ("cannot_delete", "Cannot Delete"),
        ("activate_failed", "Activate Failed"),
        ("deactivate_failed", "Deactivate Failed"),
        ("export_failed", "Export Failed"),
        ("import_failed", "Import Failed"),
        ("no_archive_selected", "No Archive Selected"),
        ("import_success", "Import Success"),
        ("export_success", "Export Success"),
        ("read_file_failed", "Read File Failed"),
        
        // Settings page
        ("label_language", "Language"),
        ("label_archive_libraries", "Archive Libraries"),
        ("label_name", "Name"),
        ("label_path", "Path"),
        ("btn_cancel", "Cancel"),
        ("btn_apply", "Apply"),
        ("dialog_add_archive_title", "Add Archive Library"),
        ("dialog_rename_archive_title", "Rename Archive Library"),
    ].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}