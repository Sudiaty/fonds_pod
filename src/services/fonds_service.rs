/// Fonds service - coordinates fonds operations with business rules
use crate::domain::models::SchemaItem;
use crate::infrastructure::persistence::{queries, establish_connection};
use crate::infrastructure::persistence::models::{Fond, FondSchema, Series};
use std::error::Error;
use std::path::Path;
use chrono::{Datelike, Local};

/// Fonds service with direct database access
pub struct FondsService;

/// Data structure for creating a new fonds
pub struct CreateFondsInput {
    /// The classification code for this fonds (e.g., "A01")
    pub classification_code: String,
    /// Optional name for the fonds
    pub name: String,
    /// Selected schema codes in order
    pub schema_codes: Vec<String>,
    /// Creation date (defaults to current date if None)
    pub created_at: Option<String>,
}

/// Result of creating a fonds
pub struct CreateFondsResult {
    pub fond_no: String,
    pub series_count: usize,
}

impl FondsService {
    /// Generate series based on fond schemas
    /// This function creates the cartesian product of all schema items
    /// 
    /// Special handling for "Year" schema:
    /// - If "Year" is in the schemas, generate year items from created_at year to current year
    /// 
    /// # Arguments
    /// * `db_path` - Path to the database
    /// * `fond_no` - The fond number to generate series for
    /// * `created_at` - The creation date in ISO 8601 format (e.g., "2020-01-01")
    /// 
    /// # Returns
    /// Number of series created
    pub fn generate_series_for_fond(
        db_path: &Path,
        fond_no: &str,
        created_at: &str,
    ) -> Result<usize, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        
        // Get fond schemas ordered by order_no
        let fond_schemas = queries::list_fond_schemas(&mut conn, fond_no)?;
        
        if fond_schemas.is_empty() {
            return Ok(0);
        }
        
        // Collect schema items for each schema
        let mut schema_items_list: Vec<Vec<SchemaItem>> = Vec::new();
        
        // Parse created_at year for Year schema special handling
        let created_year = Self::parse_year(created_at).unwrap_or(Local::now().year());
        let current_year = Local::now().year();
        
        for fond_schema in &fond_schemas {
            let schema_no = &fond_schema.schema_no;
            
            if schema_no == "Year" {
                // Special handling: generate year items from created_at to current year
                let mut year_items: Vec<SchemaItem> = Vec::new();
                for year in created_year..=current_year {
                    year_items.push(SchemaItem {
                        schema_no: "Year".to_string(),
                        item_no: year.to_string(),
                        item_name: year.to_string(),
                    });
                }
                schema_items_list.push(year_items);
            } else {
                // Normal schema: get items from database
                let items = queries::list_schema_items(&mut conn, schema_no)?;
                if items.is_empty() {
                    // Skip schemas with no items
                    continue;
                }
                schema_items_list.push(items);
            }
        }
        
        if schema_items_list.is_empty() {
            return Ok(0);
        }
        
        // Generate cartesian product
        let combinations = Self::cartesian_product(&schema_items_list);
        
        // Create series for each combination
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut created_count = 0;
        
        for combo in combinations {
            // Generate series_no by joining item_nos with "-"
            let series_no = combo.iter()
                .map(|item| item.item_no.as_str())
                .collect::<Vec<_>>()
                .join("-");
            
            // Generate series name by joining item_names with "-"
            let series_name = combo.iter()
                .map(|item| item.item_name.as_str())
                .collect::<Vec<_>>()
                .join("-");
            
            // Check if series already exists (avoid duplicates when updating)
            let full_series_no = format!("{}-{}", fond_no, series_no);
            if queries::series_exists(&mut conn, &full_series_no)? {
                continue;
            }
            
            let series = Series {
                series_no: full_series_no,
                fond_no: fond_no.to_string(),
                name: series_name,
                created_at: now.clone(),
            };
            
            queries::create_series(&mut conn, &series)?;
            created_count += 1;
        }
        
        Ok(created_count)
    }
    
    /// Create a new fonds with all associated data
    /// 
    /// This function:
    /// 1. Generates the fond number based on classification code
    /// 2. Creates the fond record
    /// 3. Creates FondSchema associations
    /// 4. Generates series based on the schemas
    pub fn create_fonds(
        db_path: &Path,
        input: CreateFondsInput,
    ) -> Result<CreateFondsResult, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        
        // 1. Generate fond number
        let seq = queries::get_next_sequence(&mut conn, &input.classification_code)?;
        let formatted_seq = queries::format_sequence(seq, 2);
        let fond_no = format!("{}{}", input.classification_code, formatted_seq);
        
        // 2. Determine created_at
        let created_at = input.created_at.unwrap_or_else(|| {
            Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
        });
        
        // 3. Create fond record
        let fond = Fond {
            fond_no: fond_no.clone(),
            fond_classification_code: input.classification_code.clone(),
            name: input.name,
            created_at: created_at.clone(),
        };
        queries::create_fond(&mut conn, &fond)?;
        
        // 4. Create FondSchema associations
        for (order, schema_code) in input.schema_codes.iter().enumerate() {
            let fond_schema = FondSchema {
                fond_no: fond_no.clone(),
                schema_no: schema_code.clone(),
                order_no: order as i32,
            };
            queries::create_fond_schema(&mut conn, &fond_schema)?;
        }
        
        // 5. Generate series (use separate function for reusability)
        drop(conn); // Release connection before calling another function that needs it
        let series_count = Self::generate_series_for_fond(db_path, &fond_no, &created_at)?;
        
        Ok(CreateFondsResult {
            fond_no,
            series_count,
        })
    }
    
    /// List all fonds in the database
    pub fn list_fonds(db_path: &Path) -> Result<Vec<Fond>, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        queries::list_fonds(&mut conn)
    }
    
    /// List all series for a fond
    pub fn list_series(db_path: &Path, fond_no: &str) -> Result<Vec<Series>, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        queries::list_series_by_fond(&mut conn, fond_no)
    }
    
    // ========================================================================
    // Helper functions
    // ========================================================================
    
    /// Parse year from ISO 8601 date string (e.g., "2020-01-01" -> 2020)
    fn parse_year(date_str: &str) -> Option<i32> {
        date_str.split('-').next()?.parse().ok()
    }
    
    /// Generate cartesian product of schema items
    /// 
    /// Example:
    /// Input: [[A, B], [1, 2]]
    /// Output: [[A, 1], [A, 2], [B, 1], [B, 2]]
    fn cartesian_product(lists: &[Vec<SchemaItem>]) -> Vec<Vec<SchemaItem>> {
        if lists.is_empty() {
            return vec![vec![]];
        }
        
        let mut result: Vec<Vec<SchemaItem>> = vec![vec![]];
        
        for list in lists {
            let mut new_result: Vec<Vec<SchemaItem>> = Vec::new();
            for combo in &result {
                for item in list {
                    let mut new_combo = combo.clone();
                    new_combo.push(item.clone());
                    new_result.push(new_combo);
                }
            }
            result = new_result;
        }
        
        result
    }

    /// List files for a series
    pub fn list_files_by_series(db_path: &Path, series_no: &str) -> Result<Vec<crate::infrastructure::persistence::models::File>, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        queries::list_files_by_series(&mut conn, series_no)
    }

    /// Delete a series
    pub fn delete_series(db_path: &Path, series_no: &str) -> Result<bool, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        queries::delete_series(&mut conn, series_no)
    }

    /// List items for a file
    pub fn list_items_by_file(db_path: &Path, file_no: &str) -> Result<Vec<crate::infrastructure::persistence::models::Item>, Box<dyn Error>> {
        let mut conn = establish_connection(db_path)?;
        queries::list_items_by_file(&mut conn, file_no)
    }
}
