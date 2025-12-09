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
                log::error!("Failed to read PO file {}: {}", po_path, e);
                return Err(e.into());
            }
        }
    } else {
        log::error!("PO file not found: {}, translations cannot be loaded", po_path);
        return Err(format!("PO file not found: {}", po_path).into());
    }
    
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

