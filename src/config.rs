/**
 * Stratus Configuration Module
 *
 * Handles stratus.json configuration file parsing and CLI overrides.
 */
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    NotFound(PathBuf),

    #[error("Failed to read configuration file: {0}")]
    ReadError(String),

    #[error("Failed to write configuration file: {0}")]
    WriteError(String),

    #[error("Failed to parse configuration file: {0}")]
    ParseError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Datasource not found: {0}")]
    DatasourceNotFound(String),

    #[error("Version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: i32, found: i32 },
}

/// Datasource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasourceConfig {
    /// Database connection URL
    pub url: String,
    /// Database schemas to manage
    #[serde(default = "default_schemas")]
    pub schemas: Vec<String>,
}

fn default_schemas() -> Vec<String> {
    vec!["public".to_string()]
}

/// Generator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    /// Code generator provider
    pub provider: Option<String>,
    /// Output directory for generated code
    pub output: Option<String>,
}

/// Schema configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaConfig {
    /// Path to schema.json file
    #[serde(default = "default_schema_path")]
    pub path: String,
}

fn default_schema_path() -> String {
    "schema/schema.json".to_string()
}

/// Migrations configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationsConfig {
    /// Path to migrations directory
    #[serde(default = "default_migrations_path")]
    pub path: String,
    /// Auto-create migrations directory
    #[serde(default = "default_auto_create")]
    pub auto_create: bool,
}

fn default_migrations_path() -> String {
    "migrations".to_string()
}

fn default_auto_create() -> bool {
    true
}

/// Main stratus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StratusConfig {
    /// Configuration version
    pub version: i32,
    /// Database datasources
    #[serde(default = "HashMap::new")]
    pub datasources: HashMap<String, DatasourceConfig>,
    /// Schema configuration
    pub schema: Option<SchemaConfig>,
    /// Migrations configuration
    pub migrations: Option<MigrationsConfig>,
    /// Generator configuration
    pub generator: Option<GeneratorConfig>,
}

impl Default for StratusConfig {
    fn default() -> Self {
        Self {
            version: 1,
            datasources: HashMap::new(),
            schema: Some(SchemaConfig::default()),
            migrations: Some(MigrationsConfig::default()),
            generator: None,
        }
    }
}

impl SchemaConfig {
    pub fn default() -> Self {
        Self {
            path: default_schema_path(),
        }
    }
}

impl MigrationsConfig {
    pub fn default() -> Self {
        Self {
            path: default_migrations_path(),
            auto_create: default_auto_create(),
        }
    }
}

/// Configuration manager
#[derive(Debug, Clone)]
pub struct ConfigManager {
    config: StratusConfig,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Load configuration from file
    pub fn load(config_path: Option<&Path>) -> Result<Self, ConfigError> {
        let path = if let Some(p) = config_path {
            p.to_path_buf()
        } else {
            PathBuf::from("stratus.json")
        };

        if !path.exists() {
            return Err(ConfigError::NotFound(path));
        }

        let content =
            std::fs::read_to_string(&path).map_err(|e| ConfigError::ReadError(e.to_string()))?;

        let config: StratusConfig =
            serde_json::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        // Validate version
        if config.version != 1 {
            return Err(ConfigError::VersionMismatch {
                expected: 1,
                found: config.version,
            });
        }

        Ok(Self {
            config,
            config_path: path,
        })
    }

    /// Create default configuration
    pub fn create_default(
        config_path: &Path,
        url: Option<&str>,
        datasource_name: &str,
    ) -> Result<Self, ConfigError> {
        let mut datasources = HashMap::new();

        if let Some(url) = url {
            datasources.insert(
                datasource_name.to_string(),
                DatasourceConfig {
                    url: url.to_string(),
                    schemas: vec!["public".to_string()],
                },
            );
        }

        let config = StratusConfig {
            version: 1,
            datasources,
            schema: Some(SchemaConfig::default()),
            migrations: Some(MigrationsConfig::default()),
            generator: None,
        };

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| ConfigError::ReadError(e.to_string()))?;
            }
        }

        // Write configuration file
        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| ConfigError::InvalidConfig(e.to_string()))?;
        std::fs::write(config_path, content).map_err(|e| ConfigError::WriteError(e.to_string()))?;

        Ok(Self {
            config,
            config_path: config_path.to_path_buf(),
        })
    }

    /// Get datasource by name
    pub fn get_datasource(&self, name: &str) -> Option<&DatasourceConfig> {
        self.config.datasources.get(name)
    }

    /// Get default datasource (first one)
    pub fn get_default_datasource(&self) -> Option<&DatasourceConfig> {
        self.config.datasources.values().next()
    }

    /// Get schema path
    pub fn get_schema_path(&self) -> PathBuf {
        let schema = self
            .config
            .schema
            .as_ref()
            .unwrap_or_else(|| self.default_schema_config());
        PathBuf::from(&schema.path)
    }

    /// Get migrations path
    pub fn get_migrations_path(&self) -> PathBuf {
        let migrations = self
            .config
            .migrations
            .as_ref()
            .unwrap_or_else(|| self.default_migrations_config());
        PathBuf::from(&migrations.path)
    }

    /// Get default schema config (borrowed)
    fn default_schema_config(&self) -> &SchemaConfig {
        // We need to store the default in a way that lives long enough
        static DEFAULT: once_cell::sync::Lazy<SchemaConfig> =
            once_cell::sync::Lazy::new(|| SchemaConfig::default());
        &DEFAULT
    }

    /// Get default migrations config (borrowed)
    fn default_migrations_config(&self) -> &MigrationsConfig {
        static DEFAULT: once_cell::sync::Lazy<MigrationsConfig> =
            once_cell::sync::Lazy::new(|| MigrationsConfig::default());
        &DEFAULT
    }

    /// Check if migrations directory should be auto-created
    pub fn migrations_auto_create(&self) -> bool {
        self.config
            .migrations
            .as_ref()
            .map(|m| m.auto_create)
            .unwrap_or(true)
    }

    /// Get generator config
    pub fn get_generator(&self) -> Option<&GeneratorConfig> {
        self.config.generator.as_ref()
    }

    /// Get all datasource names
    pub fn datasource_names(&self) -> Vec<&String> {
        self.config.datasources.keys().collect()
    }

    /// Check if configuration has any datasources
    pub fn has_datasources(&self) -> bool {
        !self.config.datasources.is_empty()
    }

    /// Get the raw configuration
    pub fn config(&self) -> &StratusConfig {
        &self.config
    }

    /// Get the config file path
    pub fn path(&self) -> &PathBuf {
        &self.config_path
    }
}

/// CLI overrides for configuration
#[derive(Debug, Default)]
pub struct ConfigOverrides {
    /// Override datasource URL
    pub url: Option<String>,
    /// Override schema path
    pub schema: Option<PathBuf>,
    /// Override migrations path
    pub migrations: Option<PathBuf>,
    /// Target datasource name
    pub datasource: Option<String>,
}

impl ConfigOverrides {
    /// Create new overrides
    pub fn new() -> Self {
        Self::default()
    }

    /// Set datasource name
    pub fn with_datasource(mut self, name: &str) -> Self {
        self.datasource = Some(name.to_string());
        self
    }

    /// Set URL override
    pub fn with_url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    /// Set schema path override
    pub fn with_schema(mut self, path: &Path) -> Self {
        self.schema = Some(path.to_path_buf());
        self
    }
}

/// Resolve configuration with CLI overrides
pub fn resolve_config(
    config: Option<&ConfigManager>,
    overrides: &ConfigOverrides,
) -> Result<ResolvedConfig, ConfigError> {
    // If no config file, use only overrides (legacy mode)
    let (url, schema_path, migrations_path) = if let Some(cfg) = config {
        let datasource = if let Some(ds_name) = &overrides.datasource {
            cfg.get_datasource(ds_name)
                .ok_or_else(|| ConfigError::DatasourceNotFound(ds_name.clone()))?
        } else {
            return Err(ConfigError::InvalidConfig(
                "Datasource must be specified. Use --datasource flag.".to_string(),
            ));
        };

        let url = overrides.url.as_ref().unwrap_or(&datasource.url);
        let schema_path = overrides
            .schema
            .clone()
            .unwrap_or_else(|| cfg.get_schema_path());
        let migrations_path = overrides
            .migrations
            .clone()
            .unwrap_or_else(|| cfg.get_migrations_path());

        (url.clone(), schema_path, migrations_path)
    } else {
        // Legacy mode: all required from CLI
        let url = overrides.url.as_ref().ok_or_else(|| {
            ConfigError::InvalidConfig(
                "Database URL required. Use --url flag or stratus.json config.".to_string(),
            )
        })?;

        (
            url.clone(),
            overrides
                .schema
                .clone()
                .unwrap_or_else(|| PathBuf::from("schema.json")),
            overrides
                .migrations
                .clone()
                .unwrap_or_else(|| PathBuf::from("migrations")),
        )
    };

    Ok(ResolvedConfig {
        url,
        schema_path,
        migrations_path,
    })
}

/// Resolved configuration for a command
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub url: String,
    pub schema_path: PathBuf,
    pub migrations_path: PathBuf,
}
