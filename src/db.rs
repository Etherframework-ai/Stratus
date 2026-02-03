/**
 * Stratus Database Operations Module
 *
 * Handles database connections, schema introspection, DDL generation, and execution.
 */
use postgres::{Client, Config, NoTls};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Database connection configuration
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Connection string (e.g., postgresql://user:pass@host:5432/db)
    pub connection_string: String,
    /// Maximum pool size (for future connection pooling)
    pub max_connections: u32,
}

/// Database connection result
pub type DbResult<T> = Result<T, DbError>;

/// Database errors
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Connection failed: {0}")]
    Connection(String),

    #[error("Query failed: {0}")]
    Query(String),

    #[error("Schema mismatch: {0}")]
    SchemaMismatch(String),

    #[error("Data loss would occur: {0}")]
    DataLoss(String),

    #[error("Migration not found: {0}")]
    MigrationNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SQL error: {0}")]
    Sql(#[from] postgres::Error),
}

/// Table column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbColumn {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub default_value: Option<String>,
    pub size: Option<usize>,
}

/// Table definition from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbTable {
    pub name: String,
    pub columns: HashMap<String, DbColumn>,
    pub primary_key: Vec<String>,
}

/// Database schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbSchema {
    pub tables: HashMap<String, DbTable>,
    pub enums: HashMap<String, Vec<String>>,
    pub dialect: String,
}

/// Database client wrapper
pub struct StratusClient {
    client: Client,
    connection_string: String,
}

impl StratusClient {
    /// Connect to database
    pub fn connect(config: &DbConfig) -> DbResult<Self> {
        let client = Client::connect(&config.connection_string, NoTls)
            .map_err(|e| DbError::Connection(e.to_string()))?;

        Ok(Self {
            client,
            connection_string: config.connection_string.clone(),
        })
    }

    /// Test connection
    pub fn ping(&mut self) -> DbResult<()> {
        self.client
            .simple_query("SELECT 1")
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(())
    }

    /// Execute DDL statement
    pub fn execute(&mut self, sql: &str) -> DbResult<()> {
        self.client
            .batch_execute(sql)
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(())
    }

    /// Execute query and return results
    pub fn query(&mut self, sql: &str) -> DbResult<Vec<HashMap<String, String>>> {
        let rows = self
            .client
            .query(sql, &[])
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut results = Vec::new();
        for row in &rows {
            let mut map = HashMap::new();
            for (i, col) in row.columns().iter().enumerate() {
                let value: Option<String> = row.get(i);
                map.insert(
                    col.name().to_string(),
                    value.unwrap_or_else(|| "NULL".to_string()),
                );
            }
            results.push(map);
        }

        Ok(results)
    }

    /// Get all tables
    pub fn get_schema(&mut self) -> DbResult<DbSchema> {
        let mut tables = HashMap::new();
        let mut enums = HashMap::new();

        // Get tables
        let rows = self.client.query(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' ORDER BY table_name",
            &[]
        ).map_err(|e| DbError::Query(e.to_string()))?;

        for row in &rows {
            let table_name: String = row.get(0);
            let columns = self.get_table_columns(&table_name)?;
            let primary_key = self.get_primary_key(&table_name)?;

            tables.insert(
                table_name.clone(),
                DbTable {
                    name: table_name.clone(),
                    columns,
                    primary_key,
                },
            );
        }

        // Get enums
        let enum_rows = self
            .client
            .query(
                "SELECT t.typname, e.enumlabel 
             FROM pg_type t 
             JOIN pg_enum e ON t.oid = e.enumtypid 
             JOIN pg_namespace n ON n.oid = t.typnamespace 
             WHERE n.nspname = 'public'
             ORDER BY t.typname, e.enumlabel",
                &[],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut current_enum = String::new();
        let mut enum_values = Vec::new();

        for row in &enum_rows {
            let type_name: String = row.get(0);
            let enum_label: String = row.get(1);

            if type_name != current_enum {
                if !current_enum.is_empty() {
                    enums.insert(current_enum.clone(), enum_values.clone());
                }
                current_enum = type_name;
                enum_values = Vec::new();
            }
            enum_values.push(enum_label);
        }

        if !current_enum.is_empty() {
            enums.insert(current_enum, enum_values);
        }

        Ok(DbSchema {
            tables,
            enums,
            dialect: "postgresql".to_string(),
        })
    }

    /// Get columns for a table
    fn get_table_columns(&mut self, table_name: &str) -> DbResult<HashMap<String, DbColumn>> {
        let rows = self.client.query(
            "SELECT column_name, data_type, is_nullable, column_default, character_maximum_length
             FROM information_schema.columns 
             WHERE table_name = $1 AND table_schema = 'public'
             ORDER BY ordinal_position",
            &[&table_name]
        ).map_err(|e| DbError::Query(e.to_string()))?;

        let mut columns = HashMap::new();
        for row in &rows {
            let name: String = row.get(0);
            let data_type: String = row.get(1);
            let is_nullable: String = row.get(2);
            let default_value: Option<String> = row.get(3);
            let size: Option<i32> = row.get(4);

            columns.insert(
                name.clone(),
                DbColumn {
                    name,
                    data_type,
                    is_nullable: is_nullable == "YES",
                    is_primary_key: false, // Will be updated separately
                    default_value,
                    size: size.map(|s| s as usize),
                },
            );
        }

        Ok(columns)
    }

    /// Get primary key columns
    fn get_primary_key(&mut self, table_name: &str) -> DbResult<Vec<String>> {
        let rows = self
            .client
            .query(
                "SELECT a.attname
             FROM pg_index i
             JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
             JOIN pg_class c ON c.oid = i.indrelid
             JOIN pg_namespace n ON n.oid = c.relnamespace
             WHERE i.indisprimary
             AND c.relname = $1
             AND n.nspname = 'public'
             ORDER BY a.attnum",
                &[&table_name],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut pk = Vec::new();
        for row in &rows {
            let name: String = row.get(0);
            pk.push(name);
        }

        Ok(pk)
    }

    /// Begin transaction
    pub fn begin(&mut self) -> DbResult<()> {
        self.execute("BEGIN")
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(())
    }

    /// Commit transaction
    pub fn commit(&mut self) -> DbResult<()> {
        self.execute("COMMIT")
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(())
    }

    /// Rollback transaction
    pub fn rollback(&mut self) -> DbResult<()> {
        self.execute("ROLLBACK")
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(())
    }
}

/// Result of schema comparison
#[derive(Debug, Default)]
pub struct SchemaDiff {
    pub create_tables: Vec<String>,
    pub alter_tables: Vec<String>,
    pub drop_tables: Vec<String>,
    pub create_columns: HashMap<String, Vec<DbColumn>>,
    pub alter_columns: HashMap<String, Vec<DbColumn>>,
    pub drop_columns: HashMap<String, Vec<String>>,
    pub create_enums: Vec<String>,
    pub drop_enums: Vec<String>,
    pub data_loss_warning: Vec<String>,
    pub sql: String,
}

impl SchemaDiff {
    pub fn has_changes(&self) -> bool {
        !self.create_tables.is_empty()
            || !self.alter_tables.is_empty()
            || !self.drop_tables.is_empty()
            || !self.create_columns.is_empty()
            || !self.alter_columns.is_empty()
            || !self.drop_columns.is_empty()
    }

    /// Calculate checksum of the SQL for deduplication
    pub fn checksum(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&self.sql);
        format!("sha256:{:x}", hasher.finalize())
    }
}

/// Generate SQL DDL from JSON schema
pub fn generate_create_table_sql(
    table_name: &str,
    table: &crate::schema::Table,
    dialect: &str,
) -> String {
    let mut sql = format!("CREATE TABLE {} (\n", table_name);

    let mut first = true;

    // Primary key first
    let pk_cols: Vec<String> = table
        .columns
        .iter()
        .filter(|(_, c)| c.is_primary_key())
        .map(|(name, _)| name.clone())
        .collect();

    if !pk_cols.is_empty() {
        sql.push_str(&format!("  PRIMARY KEY ({})\n", pk_cols.join(", ")));
        first = false;
    }

    // Other columns
    for (col_name, col) in &table.columns {
        if col.is_primary_key() {
            continue;
        }

        if !first {
            sql.push_str(",\n");
        }
        first = false;

        sql.push_str(&format!("  {}", col_name));
        sql.push_str(&format!(" {}", map_type_to_sql(&col.data_type, col.size)));

        if !col.is_not_null() {
            sql.push_str(" NULL");
        } else {
            sql.push_str(" NOT NULL");
        }

        if let Some(default) = &col.default {
            sql.push_str(&format!(" DEFAULT {}", default));
        }

        if col.generated.is_some() {
            sql.push_str(" GENERATED ALWAYS AS IDENTITY");
        }
    }

    sql.push_str("\n)");

    // Table options
    if let Some(opts) = &table.options.fillfactor {
        sql.push_str(&format!(" WITH (fillfactor = {})", opts));
    }

    sql.push_str(";");

    sql
}

/// Map JSON schema type to SQL type
fn map_type_to_sql(schema_type: &str, size: Option<usize>) -> String {
    match schema_type {
        "varchar" | "char" => {
            if let Some(s) = size {
                format!("VARCHAR({})", s)
            } else {
                "VARCHAR(255)".to_string()
            }
        }
        "decimal" => "DECIMAL(10, 2)".to_string(),
        "bigint" => "BIGINT".to_string(),
        "integer" => "INTEGER".to_string(),
        "smallint" => "SMALLINT".to_string(),
        "float" | "double" => "DOUBLE PRECISION".to_string(),
        "boolean" => "BOOLEAN".to_string(),
        "date" => "DATE".to_string(),
        "timestamp" | "timestamptz" => "TIMESTAMP WITH TIME ZONE".to_string(),
        "json" => "JSON".to_string(),
        "jsonb" => "JSONB".to_string(),
        "text" => "TEXT".to_string(),
        "uuid" => "UUID".to_string(),
        "bytea" => "BYTEA".to_string(),
        _ => schema_type.to_string(),
    }
}

/// Compare JSON schema with database schema
pub fn compare_schemas(json_schema: &crate::schema::Schema, db_schema: &DbSchema) -> SchemaDiff {
    let mut diff = SchemaDiff::default();

    // Find tables to create
    for (table_name, table) in &json_schema.tables {
        if !db_schema.tables.contains_key(table_name) {
            diff.create_tables.push(table_name.clone());
        }
    }

    // Find tables to drop
    for (table_name, _) in &db_schema.tables {
        if !json_schema.tables.contains_key(table_name) {
            diff.drop_tables.push(table_name.clone());
            diff.data_loss_warning.push(format!(
                "Table '{}' will be dropped with all data",
                table_name
            ));
        }
    }

    // Find columns to add
    for (table_name, json_table) in &json_schema.tables {
        if let Some(db_table) = db_schema.tables.get(table_name) {
            for (col_name, json_col) in &json_table.columns {
                if !db_table.columns.contains_key(col_name) {
                    diff.create_columns
                        .entry(table_name.clone())
                        .or_insert_with(Vec::new)
                        .push(DbColumn {
                            name: col_name.clone(),
                            data_type: json_col.data_type.clone(),
                            is_nullable: !json_col.is_not_null(),
                            is_primary_key: json_col.is_primary_key(),
                            default_value: json_col.default.clone(),
                            size: json_col.size,
                        });
                }
            }
        }
    }

    // Find columns to drop
    for (table_name, db_table) in &db_schema.tables {
        if let Some(json_table) = json_schema.tables.get(table_name) {
            for (col_name, _) in &db_table.columns {
                if !json_table.columns.contains_key(col_name) {
                    diff.drop_columns
                        .entry(table_name.clone())
                        .or_insert_with(Vec::new)
                        .push(col_name.clone());
                    diff.data_loss_warning.push(format!(
                        "Column '{}.{}' will be dropped",
                        table_name, col_name
                    ));
                }
            }
        }
    }

    // Generate SQL
    let mut sql = String::new();

    // Drop columns first
    for (table, columns) in &diff.drop_columns {
        for col in columns {
            sql.push_str(&format!(
                "ALTER TABLE {} DROP COLUMN IF EXISTS {};\n",
                table, col
            ));
        }
    }

    // Drop tables
    for table in &diff.drop_tables {
        sql.push_str(&format!("DROP TABLE IF EXISTS {} CASCADE;\n", table));
    }

    // Create tables
    for table_name in &diff.create_tables {
        if let Some(table) = json_schema.tables.get(table_name) {
            sql.push_str(&format!("\n-- Create table {}\n", table_name));
            sql.push_str(&generate_create_table_sql(table_name, table, "postgresql"));
            sql.push('\n');
        }
    }

    // Add columns
    for (table, columns) in &diff.create_columns {
        for col in columns {
            sql.push_str(&format!(
                "ALTER TABLE {} ADD COLUMN {} {} {};\n",
                table,
                col.name,
                map_type_to_sql(&col.data_type, col.size),
                if col.is_nullable { "NULL" } else { "NOT NULL" }
            ));
        }
    }

    diff.sql = sql;
    diff
}

/// Print schema diff summary
pub fn print_diff_summary(diff: &SchemaDiff) {
    println!();
    println!("Schema diff summary:");
    println!("{}", "=".repeat(60));

    if !diff.create_tables.is_empty() {
        println!("\nTables to CREATE ({}):", diff.create_tables.len());
        for table in &diff.create_tables {
            println!("  + {}", table);
        }
    }

    if !diff.alter_tables.is_empty() {
        println!("\nTables to ALTER ({}):", diff.alter_tables.len());
        for table in &diff.alter_tables {
            println!("  ~ {}", table);
        }
    }

    if !diff.drop_tables.is_empty() {
        println!("\nTables to DROP ({}):", diff.drop_tables.len());
        for table in &diff.drop_tables {
            println!("  - {}", table);
        }
    }

    if !diff.create_columns.is_empty() {
        println!("\nColumns to ADD ({} tables):", diff.create_columns.len());
        for (table, columns) in &diff.create_columns {
            for col in columns {
                println!("  + {}.{}", table, col.name);
            }
        }
    }

    if !diff.drop_columns.is_empty() {
        println!("\nColumns to DROP ({} tables):", diff.drop_columns.len());
        for (table, columns) in &diff.drop_columns {
            for col in columns {
                println!("  - {}.{}", table, col);
            }
        }
    }

    if !diff.data_loss_warning.is_empty() {
        println!("\n⚠️  WARNING - Data loss may occur:");
        for warning in &diff.data_loss_warning {
            println!("  ! {}", warning);
        }
    }

    if !diff.has_changes() {
        println!("\n✓ Schemas are in sync - no changes needed.");
    } else if !diff.data_loss_warning.is_empty() {
        println!("\n⚠️  Some changes may cause data loss.");
        println!("Use --accept-data-loss flag to proceed.");
    }

    println!();
}

impl DbSchema {
    /// Convert DbSchema to JSON schema format
    pub fn to_json_schema(&self) -> crate::schema::Schema {
        let mut tables = std::collections::HashMap::new();

        for (table_name, db_table) in &self.tables {
            let mut columns = std::collections::HashMap::new();

            for (col_name, db_col) in &db_table.columns {
                columns.insert(
                    col_name.clone(),
                    crate::schema::Column {
                        column_name: db_col.name.clone(),
                        data_type: db_col.data_type.clone(),
                        size: db_col.size,
                        array_dimensions: None,
                        is_primary_key: db_col.is_primary_key,
                        is_not_null: !db_col.is_nullable,
                        is_unique: false,
                        default: db_col.default_value.clone(),
                        identity: None,
                        generated: None,
                        collation: None,
                        storage: None,
                        statistics: None,
                        attributes: crate::schema::ColumnAttributes::default(),
                        references: None,
                    },
                );
            }

            tables.insert(
                table_name.clone(),
                crate::schema::Table {
                    columns,
                    indexes: None,
                    constraints: None,
                    options: crate::schema::TableOptions::default(),
                    partitions: Vec::new(),
                    inherits: Vec::new(),
                },
            );
        }

        crate::schema::Schema {
            version: Some("1".to_string()),
            dialect: Some(self.dialect.clone()),
            tables,
            enums: Some(self.enums.clone()),
        }
    }
}

impl SchemaDiff {
    /// Generate rollback SQL for the changes
    pub fn generate_rollback(&self) -> String {
        let mut sql = String::new();

        // Reverse the operations (inverse order)
        for table in &self.create_tables {
            sql.push_str(&format!("DROP TABLE IF EXISTS {} CASCADE;\n", table));
        }

        for (table, columns) in &self.create_columns {
            for col in columns {
                sql.push_str(&format!(
                    "ALTER TABLE {} DROP COLUMN IF EXISTS {};\n",
                    table, col.name
                ));
            }
        }

        for table in &self.drop_tables {
            sql.push_str(&format!(
                "-- Recreate table {} (you may need to restore from backup)\n",
                table
            ));
            sql.push_str("-- This is a placeholder - manual intervention may be required\n");
        }

        sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_column_serialization() {
        let column = DbColumn {
            name: "id".to_string(),
            data_type: "bigint".to_string(),
            is_nullable: false,
            is_primary_key: true,
            default_value: None,
            size: None,
        };

        let json = serde_json::to_string(&column).unwrap();
        assert!(json.contains("id"));
        assert!(json.contains("bigint"));
    }

    #[test]
    fn test_db_table_serialization() {
        let mut columns = std::collections::HashMap::new();
        columns.insert(
            "id".to_string(),
            DbColumn {
                name: "id".to_string(),
                data_type: "bigint".to_string(),
                is_nullable: false,
                is_primary_key: true,
                default_value: None,
                size: None,
            },
        );

        let table = DbTable {
            name: "users".to_string(),
            columns,
            primary_key: vec!["id".to_string()],
        };

        let json = serde_json::to_string(&table).unwrap();
        assert!(json.contains("users"));
        assert!(json.contains("id"));
    }

    #[test]
    fn test_db_schema_serialization() {
        let mut tables = std::collections::HashMap::new();
        tables.insert(
            "users".to_string(),
            DbTable {
                name: "users".to_string(),
                columns: std::collections::HashMap::new(),
                primary_key: vec![],
            },
        );

        let mut enums = std::collections::HashMap::new();
        enums.insert(
            "user_status".to_string(),
            vec!["active".to_string(), "inactive".to_string()],
        );

        let schema = DbSchema {
            tables,
            enums,
            dialect: "postgresql".to_string(),
        };

        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("postgresql"));
        assert!(json.contains("users"));
    }

    #[test]
    fn test_schema_diff_has_changes_empty() {
        let diff = SchemaDiff::default();
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_schema_diff_has_changes_with_tables() {
        let mut diff = SchemaDiff::default();
        diff.create_tables.push("users".to_string());
        assert!(diff.has_changes());
    }

    #[test]
    fn test_schema_diff_has_changes_with_columns() {
        let mut diff = SchemaDiff::default();
        let mut columns_map = std::collections::HashMap::new();
        columns_map.insert("users".to_string(), vec![]);
        diff.create_columns = columns_map;
        assert!(diff.has_changes());
    }

    #[test]
    fn test_db_config() {
        let config = DbConfig {
            connection_string: "postgresql://localhost/test".to_string(),
            max_connections: 5,
        };
        assert_eq!(config.max_connections, 5);
        assert!(config.connection_string.contains("localhost"));
    }
}
