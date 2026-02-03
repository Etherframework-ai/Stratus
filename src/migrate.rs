/**
 * Stratus Migration Module
 *
 * Handles migration file generation, management, and application.
 */
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn default_status() -> String {
    "draft".to_string()
}

/// Migration file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationMeta {
    /// Unique migration ID (timestamp-based)
    pub id: String,
    /// Migration name (kebab-case)
    pub name: String,
    /// When the migration was created
    pub created_at: String,
    /// Database dialect
    pub dialect: String,
    /// SHA256 checksum of the migration SQL (for deduplication)
    pub checksum: Option<String>,
    /// Migration status: draft, reviewed, applied, failed
    #[serde(default = "default_status")]
    pub status: String,
    /// Who created this migration
    pub created_by: Option<String>,
    /// When the migration was applied (if applied)
    pub applied_at: Option<String>,
}

/// Migration file
#[derive(Debug)]
pub struct Migration {
    /// Migration metadata
    pub meta: MigrationMeta,
    /// Up migration SQL (schema changes)
    pub up_sql: String,
    /// Down migration SQL (rollback)
    pub down_sql: String,
    /// Applied status
    pub applied: bool,
    /// When the migration was applied (if applied)
    pub applied_at: Option<String>,
}

/// Migration manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationManifest {
    /// All migrations
    pub migrations: Vec<MigrationMeta>,
    /// Last migration ID
    pub last_migration_id: Option<String>,
    /// Schema version
    pub schema_version: Option<String>,
}

/// Create a new migration
pub fn create_migration(
    migrations_dir: &PathBuf,
    name: &str,
    up_sql: &str,
    down_sql: &str,
    dialect: &str,
    checksum: Option<String>,
) -> Result<Migration, String> {
    // Create migrations directory if needed
    if !migrations_dir.exists() {
        fs::create_dir_all(migrations_dir)
            .map_err(|e| format!("Failed to create migrations directory: {}", e))?;
    }

    // Generate migration ID (timestamp + random)
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Time error: {}", e))?
        .as_secs();
    let random_suffix = rand::random::<u32>();
    let id = format!("{:}_{}", timestamp, random_suffix);

    // Format name (kebab-case)
    let formatted_name = name.to_lowercase().replace('_', "-").replace(' ', "-");

    // Create migration directory
    let migration_dir = migrations_dir.join(format!("{}_{}", id, formatted_name));
    fs::create_dir_all(&migration_dir)
        .map_err(|e| format!("Failed to create migration directory: {}", e))?;

    // Write up.sql
    let up_path = migration_dir.join("up.sql");
    fs::write(&up_path, up_sql).map_err(|e| format!("Failed to write up.sql: {}", e))?;

    // Write down.sql
    let down_path = migration_dir.join("down.sql");
    fs::write(&down_path, down_sql).map_err(|e| format!("Failed to write down.sql: {}", e))?;

    // Write meta.json
    let meta = MigrationMeta {
        id: id.clone(),
        name: formatted_name,
        created_at: chrono::Utc::now().to_rfc3339(),
        dialect: dialect.to_string(),
        checksum,
        status: "draft".to_string(),
        created_by: std::env::var("USER").ok(),
        applied_at: None,
    };

    let meta_path = migration_dir.join("meta.json");
    let meta_json = serde_json::to_string_pretty(&meta)
        .map_err(|e| format!("Failed to serialize meta: {}", e))?;
    fs::write(&meta_path, meta_json).map_err(|e| format!("Failed to write meta.json: {}", e))?;

    Ok(Migration {
        meta,
        up_sql: up_sql.to_string(),
        down_sql: down_sql.to_string(),
        applied: false,
        applied_at: None,
    })
}

/// Calculate SHA256 checksum of SQL content
pub fn calculate_checksum(sql: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(sql);
    format!("sha256:{:x}", hasher.finalize())
}

/// Load all migrations from directory
pub fn load_migrations(migrations_dir: &PathBuf) -> Result<Vec<Migration>, String> {
    if !migrations_dir.exists() {
        return Ok(Vec::new());
    }

    let mut migrations: Vec<Migration> = Vec::new();

    // Read directory entries
    let entries = fs::read_dir(migrations_dir)
        .map_err(|e| format!("Failed to read migrations directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Directory error: {}", e))?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        // Load meta.json
        let meta_path = path.join("meta.json");
        if !meta_path.exists() {
            continue;
        }

        let meta_json = fs::read_to_string(&meta_path)
            .map_err(|e| format!("Failed to read meta.json: {}", e))?;
        let meta: MigrationMeta = serde_json::from_str(&meta_json)
            .map_err(|e| format!("Failed to parse meta.json: {}", e))?;

        // Load up.sql
        let up_sql = if path.join("up.sql").exists() {
            fs::read_to_string(path.join("up.sql"))
                .map_err(|e| format!("Failed to read up.sql: {}", e))?
        } else {
            String::new()
        };

        // Load down.sql
        let down_sql = if path.join("down.sql").exists() {
            fs::read_to_string(path.join("down.sql"))
                .map_err(|e| format!("Failed to read down.sql: {}", e))?
        } else {
            String::new()
        };

        migrations.push(Migration {
            meta: meta.clone(),
            up_sql,
            down_sql,
            applied: false,
            applied_at: None,
        });
    }

    // Sort by ID (timestamp-based)
    migrations.sort_by(|a, b| a.meta.id.cmp(&b.meta.id));

    Ok(migrations)
}

/// Get pending migrations (not yet applied)
pub fn get_pending_migrations(migrations: &[Migration]) -> Vec<&Migration> {
    migrations.iter().filter(|m| !m.applied).collect()
}

/// Generate migration name from schema changes
pub fn generate_migration_name(from: &crate::schema::Schema, to: &crate::schema::Schema) -> String {
    let mut changes: Vec<String> = Vec::new();

    // Count new tables
    let new_tables: Vec<String> = to
        .tables
        .keys()
        .filter(|k| !from.tables.contains_key(*k))
        .map(|k| k.clone())
        .collect();

    if !new_tables.is_empty() {
        changes.push(format!("add-{}", new_tables.join("-and-")));
    }

    // Count dropped tables
    let dropped_tables: Vec<String> = from
        .tables
        .keys()
        .filter(|k| !to.tables.contains_key(*k))
        .map(|k| k.clone())
        .collect();

    if !dropped_tables.is_empty() {
        changes.push(format!("remove-{}", dropped_tables.join("-and-")));
    }

    // Generate name
    if changes.is_empty() {
        String::from("update-schema")
    } else {
        changes.join("-")
    }
}

/// Print migration status
pub fn print_migration_status(migrations: &[Migration]) {
    println!();
    println!("Migration Status");
    println!("{}", "=".repeat(50));

    let applied_count = migrations.iter().filter(|m| m.applied).count();
    let pending_count = migrations.len() - applied_count;

    println!("Total migrations: {}", migrations.len());
    println!("  ✓ Applied: {}", applied_count);
    println!("  ○ Pending: {}", pending_count);
    println!();

    if pending_count > 0 {
        println!("Pending migrations:");
        for m in migrations.iter().filter(|m| !m.applied) {
            println!("  [{}] {}", m.meta.id, m.meta.name);
        }
    } else {
        println!("✓ All migrations are up to date.");
    }

    println!();
}

/// Format SQL with basic indentation
pub fn format_sql(sql: &str) -> String {
    // Basic SQL formatting
    let mut formatted = String::new();
    let mut indent_level: i32 = 0;
    let lines: Vec<&str> = sql.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Decrease indent for closing statements
        if trimmed.starts_with(')') || trimmed.starts_with("END") || trimmed.starts_with("ALTER") {
            indent_level = indent_level.saturating_sub(1);
        }

        // Add indentation
        for _ in 0..indent_level {
            formatted.push_str("  ");
        }

        formatted.push_str(trimmed);
        formatted.push('\n');

        // Increase indent for opening statements
        if trimmed.ends_with('(') || trimmed.contains("BEGIN") {
            indent_level += 1;
        }
    }

    formatted
}

// Re-export StratusClient from db module for convenience
pub use crate::db::StratusClient;
