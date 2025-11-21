/// Classification service - coordinates fond classification operations
use crate::infrastructure::persistence::{queries, establish_connection};
use crate::infrastructure::persistence::models::FondClassification;
use std::error::Error;
use std::path::Path;

pub struct ClassificationService;

impl ClassificationService {
    /// Create top-level classification
    pub fn create_top(db_path: &Path, code: String, name: String) -> Result<(), Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        let model = FondClassification { code, name, parent_id: None, is_active: true };
        queries::create_top_classification(&mut conn, &model)
    }

    /// Create child classification under parent
    pub fn create_child(db_path: &Path, parent_code: String, code: String, name: String) -> Result<(), Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        let model = FondClassification { code, name, parent_id: Some(parent_code), is_active: true };
        queries::create_child_classification(&mut conn, &model)
    }

    /// List all top-level classifications
    pub fn list_top(db_path: &Path) -> Result<Vec<FondClassification>, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        queries::list_top_classifications(&mut conn)
    }

    /// List child classifications of parent
    pub fn list_children(db_path: &Path, parent_code: &str) -> Result<Vec<FondClassification>, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        queries::list_child_classifications(&mut conn, parent_code)
    }

    /// Delete classification (only if no children and not referenced)
    pub fn delete(db_path: &Path, class_code: &str) -> Result<bool, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        queries::delete_classification(&mut conn, class_code)
    }

    /// Activate classification
    pub fn activate(db_path: &Path, class_code: &str) -> Result<(), Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        queries::activate_classification(&mut conn, class_code)
    }

    /// Deactivate classification
    pub fn deactivate(db_path: &Path, class_code: &str) -> Result<(), Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        queries::deactivate_classification(&mut conn, class_code)
    }

    /// Export classifications to JSON
    pub fn export_to_json(db_path: &Path) -> Result<String, Box<dyn Error>> {
        use serde_json::json;
        
        let tops = Self::list_top(db_path)?;
        let mut result = Vec::new();
        
        for top in tops {
            let children = Self::list_children(db_path, &top.code)?;
            let children_json: Vec<serde_json::Value> = children.into_iter().map(|c| json!({
                "code": c.code,
                "name": c.name,
                "is_active": c.is_active
            })).collect();
            
            result.push(json!({
                "code": top.code,
                "name": top.name,
                "is_active": top.is_active,
                "children": children_json
            }));
        }
        
        Ok(serde_json::to_string_pretty(&result)?)
    }

    /// Import classifications from JSON
    pub fn import_from_json(db_path: &Path, json: &str) -> Result<(), Box<dyn Error>> {
        let data: Vec<serde_json::Value> = serde_json::from_str(json)?;
        
        for item in data {
            let code = item["code"].as_str().ok_or("Missing code")?;
            let name = item["name"].as_str().ok_or("Missing name")?;
            let is_active = item["is_active"].as_bool().unwrap_or(true);
            
            // Create top
            Self::create_top(db_path, code.to_string(), name.to_string())?;
            if !is_active {
                Self::deactivate(db_path, code)?;
            }
            
            // Create children
            if let Some(children) = item["children"].as_array() {
                for child in children {
                    let child_code = child["code"].as_str().ok_or("Missing child code")?;
                    let child_name = child["name"].as_str().ok_or("Missing child name")?;
                    let child_active = child["is_active"].as_bool().unwrap_or(true);
                    
                    Self::create_child(db_path, code.to_string(), child_code.to_string(), child_name.to_string())?;
                    if !child_active {
                        Self::deactivate(db_path, child_code)?;
                    }
                }
            }
        }
        
        Ok(())
    }
}