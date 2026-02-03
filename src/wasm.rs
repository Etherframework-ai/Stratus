//! WASM-compatible TypeSQL Parser Interface
//!
//! This module provides a WASM-bindgen interface for the TypeSQL parser,
//! enabling high-performance parsing in JavaScript/TypeScript environments.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::parser::{extract_select_columns, extract_tables_from_sql, parse, SelectColumn};

/// Parse TypeSQL content and return JSON string
///
/// # Arguments
/// * `input` - TypeSQL content with # name: comments and SQL
///
/// # Returns
/// JSON string of parsed queries or error message
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn parse_typesql(input: &str) -> Result<String, String> {
    let result = parse(input)?;
    serde_json::to_string(&result).map_err(|e| format!("JSON serialization error: {}", e))
}

/// Extract table names from SQL query
///
/// # Arguments
/// * `sql` - SQL query string
///
/// # Returns
/// JSON array of table names
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn extract_tables(sql: &str) -> Result<String, String> {
    let tables = extract_tables_from_sql(sql);
    serde_json::to_string(&tables).map_err(|e| format!("JSON serialization error: {}", e))
}

/// Extract column names from SELECT query
///
/// # Arguments
/// * `sql` - SELECT query string
///
/// # Returns
/// JSON array of column objects with table_name, column_name, is_wildcard
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn extract_columns(sql: &str) -> Result<String, String> {
    let columns: Vec<SelectColumn> = extract_select_columns(sql);
    let result: Vec<_> = columns
        .iter()
        .map(|c| {
            serde_json::json!({
                "table_name": c.table_name,
                "column_name": c.column_name,
                "is_wildcard": c.is_wildcard,
            })
        })
        .collect();
    serde_json::to_string(&result).map_err(|e| format!("JSON serialization error: {}", e))
}

/// Validate TypeSQL syntax
///
/// # Arguments
/// * `input` - TypeSQL content to validate
///
/// # Returns
/// Boolean indicating if syntax is valid
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn validate_typesql(input: &str) -> bool {
    parse(input).is_ok()
}

/// Get version info for WASM module
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
