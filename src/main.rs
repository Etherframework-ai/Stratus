use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "stratus")]
#[command(author = "Stratus Team")]
#[command(version = "0.1.0")]
#[command(about = "Multi-language TypeSQL compiler and database toolkit", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate code from TypeSQL queries
    #[command(name = "generate")]
    Generate {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(short, long, default_value = "ts")]
        language: String,
        #[arg(long)]
        schema: Option<PathBuf>,
    },

    /// Parse TypeSQL file and print AST
    #[command(name = "parse")]
    Parse {
        #[arg(short, long)]
        input: PathBuf,
    },

    /// Generate types from schema only
    #[command(name = "gen-types")]
    GenTypes {
        #[arg(short, long)]
        schema: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(short, long, default_value = "ts")]
        language: String,
    },

    /// Benchmark comparison
    #[command(name = "benchmark")]
    Benchmark,

    /// Validate schema file
    #[command(name = "validate")]
    Validate {
        #[arg(short, long)]
        schema: Option<PathBuf>,
    },

    /// Initialize stratus configuration
    #[command(name = "init")]
    Init {
        /// Datasource URL
        #[arg(short, long)]
        url: Option<String>,
        /// Datasource name
        #[arg(short, long, default_value = "primary")]
        datasource: String,
        /// Output path for stratus.json
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Sync schema to database and create migration
    #[command(name = "sync")]
    Sync {
        /// Path to schema.json
        #[arg(short, long)]
        schema: Option<PathBuf>,
        /// Migration name (auto-generated if not provided)
        #[arg(short, long)]
        name: Option<String>,
        /// Force re-apply existing migrations
        #[arg(long)]
        force: bool,
        /// Skip applying to database (generate only)
        #[arg(long)]
        dry_run: bool,
        /// Target datasource from stratus.json
        #[arg(short, long)]
        datasource: Option<String>,
        /// Database connection string (overrides stratus.json)
        #[arg(short, long)]
        url: Option<String>,
    },

    /// ==================== Deploy Command ====================
    /// Deploy pending migrations to database
    #[command(name = "deploy")]
    Deploy {
        /// Path to schema.json
        #[arg(short, long)]
        schema: Option<PathBuf>,
        /// Target environment (staging/production)
        #[arg(short, long, value_name = "ENV")]
        env: Option<String>,
        /// Skip confirmation
        #[arg(long)]
        yes: bool,
        /// Target datasource from stratus.json
        #[arg(short, long)]
        datasource: Option<String>,
        /// Database connection string (overrides stratus.json)
        #[arg(short, long)]
        url: Option<String>,
    },

    /// ==================== Database Commands ====================
    /// Push schema state to database (prototype mode)
    #[command(name = "db")]
    Db {
        #[command(subcommand)]
        command: DbCommands,
    },

    /// ==================== Migration Commands ====================
    /// Database migrations
    #[command(name = "migrate")]
    Migrate {
        #[command(subcommand)]
        command: MigrateCommands,
    },
}

#[derive(Subcommand, Debug)]
enum DbCommands {
    /// Push schema state to database (prototype mode)
    #[command(name = "push")]
    DbPush {
        /// Path to schema.json
        #[arg(short, long)]
        schema: Option<PathBuf>,
        /// Skip code generation
        #[arg(long)]
        skip_generate: bool,
        /// Accept data loss
        #[arg(long)]
        accept_data_loss: bool,
        /// Force reset database
        #[arg(long)]
        force_reset: bool,
        /// Database connection string
        #[arg(short, long)]
        url: Option<String>,
    },

    /// Pull schema from database to schema.json
    #[command(name = "pull")]
    DbPull {
        /// Output path for schema.json
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Database connection string
        #[arg(short, long)]
        url: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum MigrateCommands {
    /// Create and apply migrations during development
    #[command(name = "dev")]
    MigrateDev {
        /// Path to schema.json
        #[arg(short, long)]
        schema: Option<PathBuf>,
        /// Migration name
        #[arg(short, long)]
        name: Option<String>,
        /// Skip code generation
        #[arg(long)]
        skip_generate: bool,
        /// Create empty migration (no schema changes)
        #[arg(long)]
        create_only: bool,
        /// Database connection string
        #[arg(short, long)]
        url: Option<String>,
    },

    /// Apply pending migrations to database
    #[command(name = "deploy")]
    MigrateDeploy {
        /// Path to schema.json
        #[arg(short, long)]
        schema: Option<PathBuf>,
        /// Database connection string
        #[arg(short, long)]
        url: Option<String>,
    },

    /// Reset database and re-apply all migrations
    #[command(name = "reset")]
    MigrateReset {
        /// Path to schema.json
        #[arg(short, long)]
        schema: Option<PathBuf>,
        /// Skip confirmation
        #[arg(long)]
        force: bool,
        /// Skip seed
        #[arg(long)]
        skip_seed: bool,
        /// Database connection string
        #[arg(short, long)]
        url: Option<String>,
    },

    /// Check migration status
    #[command(name = "status")]
    MigrateStatus {
        /// Path to schema.json
        #[arg(short, long)]
        schema: Option<PathBuf>,
    },

    /// Show the difference between two schemas
    #[command(name = "diff")]
    MigrateDiff {
        /// From schema (current database or file)
        #[arg(short, long, value_name = "SCHEMA")]
        from: Option<String>,
        /// To schema (target schema file)
        #[arg(short, long, value_name = "SCHEMA")]
        to: Option<PathBuf>,
        /// Database connection string
        #[arg(short, long)]
        url: Option<String>,
        /// Save to migration file
        #[arg(long)]
        save: bool,
        /// Migration name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Resolve migration issues
    #[command(name = "resolve")]
    MigrateResolve {
        /// Issue to resolve
        #[arg(short, long)]
        issue: String,
        /// Migration ID
        #[arg(short, long)]
        migration: Option<String>,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        // ==================== Generate ====================
        Commands::Generate {
            input,
            output,
            language,
            schema,
        } => {
            let input_str = fs::read_to_string(&input).expect("Failed to read input file");
            let ast = stratus::parser::parse(&input_str).expect("Failed to parse");

            let schema_data = schema.as_ref().map(|s| {
                let schema_str = fs::read_to_string(s).expect("Failed to read schema");
                serde_json::from_str(&schema_str).expect("Failed to parse schema")
            });

            let output_str = match language.as_str() {
                "ts" | "typescript" => stratus::codegen::generate_ts(&ast, schema_data.as_ref()),
                "py" | "python" => stratus::codegen::generate_py(&ast, schema_data.as_ref()),
                "sql" => stratus::codegen::generate_sql(&ast),
                _ => panic!("Unsupported language: {}", language),
            };

            match output {
                Some(path) => {
                    fs::write(&path, &output_str).expect("Failed to write output");
                    println!("Generated {} -> {}", language, path.display());
                }
                None => {
                    print!("{}", output_str);
                }
            }
        }

        // ==================== Parse ====================
        Commands::Parse { input } => {
            let input_str = fs::read_to_string(&input).expect("Failed to read input file");
            let ast = stratus::parser::parse(&input_str).expect("Failed to parse");
            println!("{:#?}", ast);
        }

        // ==================== Gen Types ====================
        Commands::GenTypes {
            schema,
            output,
            language,
        } => {
            let schema_str = fs::read_to_string(&schema).expect("Failed to read schema");
            let schema: stratus::schema::Schema =
                serde_json::from_str(&schema_str).expect("Failed to parse schema");

            let output_str = match language.as_str() {
                "ts" | "typescript" => stratus::codegen::generate_ts_types_only(&schema),
                "py" | "python" => stratus::codegen::generate_py_types_only(&schema),
                _ => panic!("Unsupported language: {}", language),
            };

            match output {
                Some(path) => {
                    fs::write(&path, &output_str).expect("Failed to write output");
                    println!("Generated types -> {}", path.display());
                }
                None => {
                    print!("{}", output_str);
                }
            }
        }

        // ==================== Benchmark ====================
        Commands::Benchmark => {
            println!("Running benchmark comparison...");
            println!("Comparing Prisma/Drizzle vs Stratus compile-time SQL generation");
            println!();
            println!("This would measure:");
            println!("  - Query execution time");
            println!("  - Type checking overhead");
            println!("  - Bundle size impact");
            println!();
            println!("TODO: Implement full benchmark suite");
        }

        // ==================== Validate ====================
        Commands::Validate { schema } => {
            let schema_path = schema.unwrap_or_else(|| PathBuf::from("schema.json"));
            let schema_str = match fs::read_to_string(&schema_path) {
                Ok(s) => s,
                Err(_) => {
                    eprintln!(
                        "Error: Could not read schema file: {}",
                        schema_path.display()
                    );
                    std::process::exit(1);
                }
            };

            let parsed: serde_json::Value = match serde_json::from_str(&schema_str) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Error: Invalid JSON - {}", e);
                    std::process::exit(1);
                }
            };

            // Basic structure validation
            if let Some(obj) = parsed.as_object() {
                let mut errors: Vec<String> = Vec::new();

                if !obj.contains_key("version") {
                    errors.push("Missing required field: 'version'".to_string());
                }
                if !obj.contains_key("tables") {
                    errors.push("Missing required field: 'tables'".to_string());
                } else if let Some(tables) = obj.get("tables").and_then(|t| t.as_object()) {
                    for (table_name, table) in tables {
                        if let Some(cols) = table.get("columns").and_then(|c| c.as_object()) {
                            for (col_name, col) in cols {
                                if !col.is_object() {
                                    errors.push(format!(
                                        "Table '{}' column '{}' must be an object",
                                        table_name, col_name
                                    ));
                                }
                            }
                        }
                    }
                }

                if errors.is_empty() {
                    println!("✓ Schema is valid: {}", schema_path.display());
                    println!(
                        "  Version: {:?}",
                        obj.get("version").and_then(|v| v.as_str())
                    );
                    println!(
                        "  Tables: {}",
                        obj.get("tables")
                            .map(|t| t.as_object().map(|o| o.len()).unwrap_or(0))
                            .unwrap_or(0)
                    );
                    if let Some(enums) = obj.get("enums").and_then(|e| e.as_object()) {
                        println!("  Enums: {}", enums.len());
                    }
                } else {
                    eprintln!("Error: Schema validation failed");
                    for error in &errors {
                        eprintln!("  - {}", error);
                    }
                    std::process::exit(1);
                }
            }
        }

        // ==================== Init Command ====================
        Commands::Init {
            url,
            datasource,
            output,
        } => {
            let config_path = output.unwrap_or_else(|| PathBuf::from("stratus.json"));

            println!("\n🚀  Stratus Init");
            println!("{}", "=".repeat(50));
            println!("Output: {}", config_path.display());
            println!("Datasource: {}", datasource);
            if let Some(ref url) = url {
                println!("URL: {}", url);
            } else {
                println!("URL: (not specified, edit stratus.json to add)");
            }
            println!();

            match stratus::config::ConfigManager::create_default(
                &config_path,
                url.as_deref(),
                &datasource,
            ) {
                Ok(_) => {
                    println!("✓ Created stratus.json configuration");
                    println!();
                    println!("Next steps:");
                    println!("  1. Edit stratus.json to configure database URL");
                    println!("  2. Create your schema.json in the schema/ directory");
                    println!("  3. Run: stratus sync --datasource {}", datasource);
                }
                Err(e) => {
                    eprintln!("Error creating configuration: {}", e);
                    std::process::exit(1);
                }
            }
        }

        // ==================== Sync Command ====================
        Commands::Sync {
            schema: schema_override,
            name,
            force,
            dry_run,
            datasource: datasource_override,
            url: url_override,
        } => {
            // Try to load configuration
            let config = stratus::config::ConfigManager::load(None).ok();

            // Determine schema path
            let schema_path = if let Some(ref s) = schema_override {
                s.clone()
            } else if let Some(ref cfg) = config {
                cfg.get_schema_path()
            } else {
                PathBuf::from("schema.json")
            };

            // Determine migrations directory
            let migrations_dir = if let Some(ref cfg) = config {
                cfg.get_migrations_path()
            } else {
                PathBuf::from("migrations")
            };

            // Determine database URL
            let db_url = if let Some(ds_name) = &datasource_override {
                if let Some(ref cfg) = config {
                    let ds = cfg.get_datasource(ds_name).unwrap_or_else(|| {
                        eprintln!("Error: Datasource '{}' not found in stratus.json", ds_name);
                        std::process::exit(1);
                    });
                    url_override.clone().unwrap_or(ds.url.clone())
                } else {
                    url_override.clone().unwrap_or_else(|| {
                        eprintln!(
                            "Error: stratus.json not found. Use --url or create stratus.json"
                        );
                        std::process::exit(1);
                    })
                }
            } else if let Some(ref url) = url_override {
                url.clone()
            } else if let Some(ref _cfg) = config {
                // When using stratus.json, datasource is required
                eprintln!("Error: Datasource required. Use --datasource flag or configure in stratus.json");
                std::process::exit(1);
            } else {
                std::env::var("DATABASE_URL").ok().unwrap_or_else(|| {
                    eprintln!(
                        "Error: No database URL provided. Use --url or set DATABASE_URL env var."
                    );
                    std::process::exit(1);
                })
            };

            println!("\n🔄  Stratus Sync");
            println!("{}", "=".repeat(50));
            println!("Schema: {}", schema_path.display());
            println!("Migrations: {}", migrations_dir.display());
            if let Some(ref ds) = datasource_override {
                println!("Datasource: {}", ds);
            }
            if url_override.is_some() {
                println!("URL: (CLI override)");
            }
            println!();

            // Load schema
            if !schema_path.exists() {
                eprintln!("Error: Schema file not found: {}", schema_path.display());
                std::process::exit(1);
            }
            let schema_str = fs::read_to_string(&schema_path).expect("Failed to read schema file");
            let parsed_schema: stratus::schema::Schema =
                serde_json::from_str(&schema_str).expect("Failed to parse schema");

            // Connect to database
            println!("Connecting to database...");
            let db_config = stratus::db::DbConfig {
                connection_string: db_url.clone(),
                max_connections: 5,
            };

            let mut client = match stratus::db::StratusClient::connect(&db_config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error: Failed to connect to database: {}", e);
                    std::process::exit(1);
                }
            };
            println!("Connected successfully.");
            println!();

            // Load existing migrations
            let existing_migrations = stratus::migrate::load_migrations(&migrations_dir)
                .expect("Failed to load migrations");

            // Introspect current database schema
            println!("Introspecting database schema...");
            let db_schema = match client.get_schema() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: Failed to introspect database: {}", e);
                    std::process::exit(1);
                }
            };

            // Calculate diff
            let diff = stratus::db::compare_schemas(&parsed_schema, &db_schema);
            stratus::db::print_diff_summary(&diff);

            if !diff.has_changes() {
                println!("✓ Database is in sync with schema.json");
                return;
            }

            // Check for existing migrations with same checksum
            let diff_checksum = diff.checksum();
            if !force {
                for m in &existing_migrations {
                    if m.meta.checksum == Some(diff_checksum.clone()) {
                        println!(
                            "\n⚠️  Migration already exists with same changes: {}",
                            m.meta.name
                        );
                        println!("   Use --force to re-apply");
                        return;
                    }
                }
            }

            // Check for conflicts with existing migrations
            let mut potential_conflicts = Vec::new();
            for m in &existing_migrations {
                // Check if this migration affects the same tables
                let migration_affects_tables = diff
                    .create_tables
                    .iter()
                    .any(|table| m.up_sql.contains(table))
                    || diff
                        .drop_tables
                        .iter()
                        .any(|table| m.up_sql.contains(table));

                if migration_affects_tables {
                    potential_conflicts.push(m.meta.name.clone());
                }
            }

            if !potential_conflicts.is_empty() {
                println!("\n⚠️  Potential conflicts detected!");
                println!("   These existing migrations affect similar tables:");
                for conflict in &potential_conflicts {
                    println!("   - {}", conflict);
                }
                println!();
                println!("   The new migration will be created with combined changes.");
                println!("   Please review and merge if necessary.");
                println!();
            }

            // Generate migration name
            let migration_name = name.unwrap_or_else(|| {
                stratus::migrate::generate_migration_name(
                    &db_schema.to_json_schema(),
                    &parsed_schema,
                )
            });

            // Generate up/down SQL
            let up_sql = diff.sql.clone();
            let down_sql = diff.generate_rollback();

            // Create migration
            match stratus::migrate::create_migration(
                &migrations_dir,
                &migration_name,
                &up_sql,
                &down_sql,
                "postgresql",
                Some(diff_checksum),
            ) {
                Ok(m) => {
                    println!();
                    println!("✓ Created migration: {}_{}", m.meta.id, m.meta.name);
                    println!(
                        "  File: {}/{}_{}/up.sql",
                        migrations_dir.display(),
                        m.meta.id,
                        m.meta.name
                    );
                    println!(
                        "  File: {}/{}_{}/down.sql",
                        migrations_dir.display(),
                        m.meta.id,
                        m.meta.name
                    );
                    println!("  Status: draft (editable until applied)");
                }
                Err(e) => {
                    eprintln!("Error creating migration: {}", e);
                    std::process::exit(1);
                }
            }

            if dry_run {
                println!("\n[DRY RUN] Skipping database application");
                return;
            }

            // Apply migration
            println!();
            println!("Applying migration...");

            // Use transaction for atomicity
            client.begin().expect("Failed to begin transaction");

            match client.execute(&up_sql) {
                Ok(_) => {
                    client.commit().expect("Failed to commit");
                    println!("✓ Applied migration successfully");
                }
                Err(e) => {
                    let _ = client.rollback();
                    eprintln!("\n✗ Error applying migration: {}", e);
                    std::process::exit(1);
                }
            }

            println!();
            println!("Next steps:");
            println!(
                "  1. Review migration files in: {}",
                migrations_dir.display()
            );
            println!("  2. Edit up.sql/down.sql if needed");
            println!("  3. Commit and create PR for team review");
            println!("  4. After PR merge, run: stratus deploy");
        }

        // ==================== Deploy Command ====================
        Commands::Deploy {
            schema: schema_override,
            env,
            yes,
            datasource: datasource_override,
            url: url_override,
        } => {
            // Try to load configuration
            let config = stratus::config::ConfigManager::load(None).ok();

            // Determine schema path
            let schema_path = if let Some(ref s) = schema_override {
                s.clone()
            } else if let Some(ref cfg) = config {
                cfg.get_schema_path()
            } else {
                PathBuf::from("schema.json")
            };

            // Determine migrations directory
            let migrations_dir = if let Some(ref cfg) = config {
                cfg.get_migrations_path()
            } else {
                PathBuf::from("migrations")
            };

            // Determine database URL
            let db_url = if let Some(ds_name) = &datasource_override {
                if let Some(ref cfg) = config {
                    let ds = cfg.get_datasource(ds_name).unwrap_or_else(|| {
                        eprintln!("Error: Datasource '{}' not found in stratus.json", ds_name);
                        std::process::exit(1);
                    });
                    url_override.clone().unwrap_or(ds.url.clone())
                } else {
                    url_override.clone().unwrap_or_else(|| {
                        eprintln!(
                            "Error: stratus.json not found. Use --url or create stratus.json"
                        );
                        std::process::exit(1);
                    })
                }
            } else if let Some(ref url) = url_override {
                url.clone()
            } else if let Some(ref _cfg) = config {
                eprintln!("Error: Datasource required. Use --datasource flag or configure in stratus.json");
                std::process::exit(1);
            } else {
                std::env::var("DATABASE_URL").ok().unwrap_or_else(|| {
                    eprintln!(
                        "Error: No database URL provided. Use --url or set DATABASE_URL env var."
                    );
                    std::process::exit(1);
                })
            };

            let env_name = env.unwrap_or_else(|| "unknown".to_string());
            println!("\n🚀  Stratus Deploy");
            println!("{}", "=".repeat(50));
            println!("Environment: {}", env_name);
            println!("Schema: {}", schema_path.display());
            println!("Migrations: {}", migrations_dir.display());
            if let Some(ref ds) = datasource_override {
                println!("Datasource: {}", ds);
            }
            if url_override.is_some() {
                println!("URL: (CLI override)");
            }
            println!();

            // Load migrations
            let migrations = stratus::migrate::load_migrations(&migrations_dir)
                .expect("Failed to load migrations");

            // Filter pending migrations (draft or reviewed, not applied)
            let pending_migrations: Vec<&stratus::migrate::Migration> = migrations
                .iter()
                .filter(|m| !m.applied && m.meta.status != "failed")
                .collect();

            if pending_migrations.is_empty() {
                println!("✓ No pending migrations to apply.");
                return;
            }

            println!("Found {} pending migrations:", pending_migrations.len());
            for m in &pending_migrations {
                let status = if m.meta.status == "reviewed" {
                    "✓ reviewed"
                } else {
                    "○ draft"
                };
                println!("  [{}] {} {}", m.meta.id, m.meta.name, status);
            }
            println!();

            // For production, require --yes or manual confirmation
            let is_production = env_name.to_lowercase() == "production";
            if is_production && !yes {
                println!("⚠️  This is a PRODUCTION deployment!");
                println!();
                println!("To confirm, run with --yes flag:");
                println!("  stratus deploy --env=production --yes");
                std::process::exit(1);
            }

            // Connect to database
            println!("Connecting to database...");
            let db_config = stratus::db::DbConfig {
                connection_string: db_url.clone(),
                max_connections: 5,
            };

            let mut client = match stratus::migrate::StratusClient::connect(&db_config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error: Failed to connect to database: {}", e);
                    std::process::exit(1);
                }
            };
            println!("Connected successfully.");
            println!();

            // Apply migrations in transaction
            println!("Applying migrations...");

            let mut applied_count = 0;
            let mut failed = false;

            for m in pending_migrations {
                print!("  [{}] {}... ", m.meta.id, m.meta.name);

                // Begin transaction for each migration
                client.begin().expect("Failed to begin transaction");

                match client.execute(&m.up_sql) {
                    Ok(_) => {
                        client.commit().expect("Failed to commit");
                        println!("OK");
                        applied_count += 1;
                    }
                    Err(e) => {
                        let _ = client.rollback();
                        println!("FAILED");
                        eprintln!("\n✗ Error applying migration {}: {}", m.meta.name, e);
                        failed = true;
                        break;
                    }
                }
            }

            println!();

            if failed {
                eprintln!("✗ Deployment failed!");
                eprintln!("   Some migrations were not applied.");
                eprintln!("   Check the errors above and resolve manually.");
                std::process::exit(1);
            }

            println!("✓ Successfully applied {} migration(s)", applied_count);
            println!();
            println!("Next steps:");
            println!("  1. Verify the application works correctly");
            println!("  2. Monitor logs for any issues");
            if is_production {
                println!("  3. Notify team of successful deployment");
            }
        }

        // ==================== DB Push ====================
        Commands::Db { command } => {
            match command {
                DbCommands::DbPush {
                    schema,
                    skip_generate: _,
                    accept_data_loss,
                    force_reset,
                    url,
                } => {
                    let schema_path = schema.unwrap_or_else(|| PathBuf::from("schema.json"));
                    let schema_str =
                        fs::read_to_string(&schema_path).expect("Failed to read schema file");
                    let parsed_schema: stratus::schema::Schema =
                        serde_json::from_str(&schema_str).expect("Failed to parse schema");

                    println!("\n🌱  DB Push");
                    println!("{}", "=".repeat(50));
                    println!("Schema: {}", schema_path.display());
                    println!("Tables: {}", parsed_schema.tables.len());
                    println!();

                    // Get database URL
                    let db_url = url.or_else(|| std::env::var("DATABASE_URL").ok());
                    if db_url.is_none() {
                        eprintln!("Error: No database URL provided. Use --url or set DATABASE_URL env var.");
                        std::process::exit(1);
                    }
                    let db_url = db_url.unwrap();

                    println!("Connecting to database...");
                    let db_config = stratus::db::DbConfig {
                        connection_string: db_url.clone(),
                        max_connections: 5,
                    };

                    let mut client = match stratus::db::StratusClient::connect(&db_config) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("Error: Failed to connect to database: {}", e);
                            std::process::exit(1);
                        }
                    };

                    println!("Connected successfully.");
                    println!();

                    // Force reset mode - drop all tables and recreate
                    if force_reset {
                        println!("⚠️  Force reset mode - dropping all tables!");
                        println!();

                        // Drop all existing tables
                        for (table_name, _) in &parsed_schema.tables {
                            let drop_sql = format!("DROP TABLE IF EXISTS {} CASCADE;", table_name);
                            print!("  Dropping {}... ", table_name);
                            if let Err(e) = client.execute(&drop_sql) {
                                println!("FAILED: {}", e);
                            } else {
                                println!("OK");
                            }
                        }
                        println!();
                    }

                    // Get current database schema
                    println!("Introspecting current database schema...");
                    let db_schema = match client.get_schema() {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Error: Failed to introspect database: {}", e);
                            std::process::exit(1);
                        }
                    };
                    println!("Found {} tables in database.", db_schema.tables.len());
                    println!();

                    // Compare schemas
                    let diff = stratus::db::compare_schemas(&parsed_schema, &db_schema);
                    stratus::db::print_diff_summary(&diff);

                    if !diff.has_changes() {
                        println!("✓ Database schema is in sync.");
                        return;
                    }

                    // Check for data loss
                    if !diff.data_loss_warning.is_empty() && !accept_data_loss {
                        println!("\n⚠️  Data loss would occur!");
                        println!("Use --accept-data-loss to proceed anyway.");
                        std::process::exit(1);
                    }

                    // Execute DDL
                    if diff.sql.is_empty() {
                        println!("No DDL to execute.");
                        return;
                    }

                    // Check for data loss
                    if !diff.data_loss_warning.is_empty() && !accept_data_loss {
                        println!("\n⚠️  Data loss would occur!");
                        println!("Use --accept-data-loss to proceed anyway.");
                        std::process::exit(1);
                    }

                    // Execute DDL
                    if diff.sql.is_empty() {
                        println!("No DDL to execute.");
                        return;
                    }

                    // Check for data loss
                    if !diff.data_loss_warning.is_empty() && !accept_data_loss {
                        println!("\n⚠️  Data loss would occur!");
                        println!("Use --accept-data-loss to proceed anyway.");
                        std::process::exit(1);
                    }

                    // Execute DDL
                    if diff.sql.is_empty() {
                        println!("No DDL to execute.");
                        return;
                    }

                    println!("\n🚀  Executing DDL...");
                    println!("{}", "-".repeat(50));

                    // Execute in transaction
                    client.begin().expect("Failed to begin transaction");

                    match client.execute(&diff.sql) {
                        Ok(_) => {
                            client.commit().expect("Failed to commit");
                            println!("\n✓ Successfully pushed schema to database.");
                        }
                        Err(e) => {
                            let _ = client.rollback();
                            eprintln!("\n✗ Error executing DDL: {}", e);
                            std::process::exit(1);
                        }
                    }

                    println!();
                    println!("Tables created/updated:");
                    for table in &diff.create_tables {
                        println!("  + {}", table);
                    }
                    for (table, columns) in &diff.create_columns {
                        for col in columns {
                            println!("  + {}.{}", table, col.name);
                        }
                    }
                }

                DbCommands::DbPull { output, url } => {
                    let output_path = output.unwrap_or_else(|| PathBuf::from("schema.json"));

                    println!("\n🔄  DB Pull");
                    println!("{}", "=".repeat(50));
                    println!("Output: {}", output_path.display());

                    // Get database URL
                    let db_url = url.or_else(|| std::env::var("DATABASE_URL").ok());
                    if db_url.is_none() {
                        eprintln!("Error: No database URL provided. Use --url or set DATABASE_URL env var.");
                        std::process::exit(1);
                    }
                    let db_url = db_url.unwrap();

                    println!("Connecting to database...");
                    let db_config = stratus::db::DbConfig {
                        connection_string: db_url.clone(),
                        max_connections: 5,
                    };

                    let mut client = match stratus::db::StratusClient::connect(&db_config) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("Error: Failed to connect to database: {}", e);
                            std::process::exit(1);
                        }
                    };

                    println!("Connected successfully.");
                    println!();

                    // Introspect schema
                    println!("Introspecting database schema...");
                    let db_schema = match client.get_schema() {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Error: Failed to introspect database: {}", e);
                            std::process::exit(1);
                        }
                    };

                    // Convert to JSON schema format
                    let json_schema = serde_json::to_string_pretty(&db_schema)
                        .expect("Failed to serialize schema");

                    fs::write(&output_path, &json_schema).expect("Failed to write schema file");

                    println!("✓ Pulled schema from database.");
                    println!();
                    println!("Found {} tables:", db_schema.tables.len());
                    for (table_name, table) in &db_schema.tables {
                        println!("  + {} ({} columns)", table_name, table.columns.len());
                    }

                    if !db_schema.enums.is_empty() {
                        println!();
                        println!("Found {} enums:", db_schema.enums.len());
                        for (enum_name, values) in &db_schema.enums {
                            println!("  + {} = {:?}", enum_name, values);
                        }
                    }
                }
            }
        }

        // ==================== Migrate ====================
        Commands::Migrate { command } => match command {
            MigrateCommands::MigrateDev {
                schema,
                name,
                skip_generate: _,
                create_only,
                url,
            } => {
                let schema_path = schema.unwrap_or_else(|| PathBuf::from("schema.json"));
                let migrations_dir = PathBuf::from("migrations");

                println!("\n🛠️  Migrate Dev");
                println!("{}", "=".repeat(50));
                println!("Schema: {}", schema_path.display());
                println!("Migrations: {}", migrations_dir.display());
                println!();

                // Get database URL
                let db_url = url.or_else(|| std::env::var("DATABASE_URL").ok());
                let db_config = if let Some(url) = db_url {
                    Some(stratus::db::DbConfig {
                        connection_string: url,
                        max_connections: 5,
                    })
                } else {
                    None
                };

                // Load schema
                let schema_str =
                    fs::read_to_string(&schema_path).expect("Failed to read schema file");
                let parsed_schema: stratus::schema::Schema =
                    serde_json::from_str(&schema_str).expect("Failed to parse schema");

                // Load existing migrations
                let existing_migrations = stratus::migrate::load_migrations(&migrations_dir)
                    .expect("Failed to load migrations");

                println!("Existing migrations: {}", existing_migrations.len());

                // Show status
                stratus::migrate::print_migration_status(&existing_migrations);

                // If create_only flag, just create an empty migration
                if create_only {
                    let migration_name = name.unwrap_or_else(|| "empty-migration".to_string());
                    let up_sql = "-- Empty migration\n-- Add your SQL here";
                    let down_sql = "-- Empty migration rollback";

                    match stratus::migrate::create_migration(
                        &migrations_dir,
                        &migration_name,
                        up_sql,
                        down_sql,
                        "postgresql",
                        None,
                    ) {
                        Ok(m) => {
                            println!("✓ Created empty migration: {}_{}", m.meta.id, m.meta.name);
                        }
                        Err(e) => {
                            eprintln!("Error creating migration: {}", e);
                        }
                    }
                    return;
                }

                // Need database connection for full migration workflow
                if db_config.is_none() {
                    eprintln!(
                        "Error: No database URL provided. Use --url or set DATABASE_URL env var."
                    );
                    eprintln!("For dev mode, a database connection is required.");
                    std::process::exit(1);
                }

                let mut db_config = db_config.unwrap();
                let mut client = match stratus::db::StratusClient::connect(&db_config) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error: Failed to connect to database: {}", e);
                        std::process::exit(1);
                    }
                };

                println!("Connected to database.");
                println!();

                // Introspect current database schema
                println!("Introspecting current database schema...");
                let db_schema = match client.get_schema() {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error: Failed to introspect database: {}", e);
                        std::process::exit(1);
                    }
                };

                // Compare schemas
                let diff = stratus::db::compare_schemas(&parsed_schema, &db_schema);
                stratus::db::print_diff_summary(&diff);

                if !diff.has_changes() {
                    println!("✓ Database schema is in sync. No migration needed.");
                    return;
                }

                // Generate migration name
                let migration_name = name.unwrap_or_else(|| {
                    stratus::migrate::generate_migration_name(
                        &db_schema.to_json_schema(),
                        &parsed_schema,
                    )
                });

                // Create migration
                let down_sql = format!(
                    "-- Rollback for {}\n{}",
                    migration_name,
                    diff.generate_rollback()
                );

                match stratus::migrate::create_migration(
                    &migrations_dir,
                    &migration_name,
                    &diff.sql,
                    &down_sql,
                    "postgresql",
                    None,
                ) {
                    Ok(m) => {
                        println!();
                        println!("✓ Created migration: {}_{}", m.meta.id, m.meta.name);
                        println!(
                            "  File: {}/{}_{}/up.sql",
                            migrations_dir.display(),
                            m.meta.id,
                            m.meta.name
                        );
                    }
                    Err(e) => {
                        eprintln!("Error creating migration: {}", e);
                        std::process::exit(1);
                    }
                }

                // Apply pending migrations
                println!();
                println!("Applying pending migrations...");
                let updated_migrations = stratus::migrate::load_migrations(&migrations_dir)
                    .expect("Failed to reload migrations");

                for migration in updated_migrations.iter().filter(|m| !m.applied) {
                    print!("  Applying {}... ", migration.meta.name);
                    match client.execute(&migration.up_sql) {
                        Ok(_) => {
                            println!("OK");
                        }
                        Err(e) => {
                            println!("FAILED: {}", e);
                            eprintln!("Error applying migration: {}", e);
                            std::process::exit(1);
                        }
                    }
                }

                println!();
                println!("✓ Migration complete.");
            }

            MigrateCommands::MigrateDeploy { schema: _, url: _ } => {
                println!("\n🚀  Migrate Deploy");
                println!("{}", "=".repeat(50));
                println!("Applying pending migrations to database...");
                println!();
                println!("TODO: Implement migration deployment");
            }

            MigrateCommands::MigrateReset {
                schema,
                force: _,
                skip_seed: _,
                url: _,
            } => {
                let schema_path = schema.unwrap_or_else(|| PathBuf::from("schema.json"));
                let migrations_dir = PathBuf::from("migrations");

                println!("\n⚠️  Migrate Reset");
                println!("{}", "=".repeat(50));
                println!("This will:");
                println!("  1. Drop all tables in the database");
                println!("  2. Re-create all tables from migrations");
                println!("  3. ALL DATA WILL BE LOST");
                println!();
                println!("Schema: {}", schema_path.display());
                println!("Migrations: {}", migrations_dir.display());
                println!();
                println!("Use --force to skip confirmation");
            }

            MigrateCommands::MigrateStatus { schema: _ } => {
                let migrations_dir = PathBuf::from("migrations");

                println!("\n📊  Migrate Status");
                println!("{}", "=".repeat(50));
                println!("Migrations: {}", migrations_dir.display());
                println!();

                let migrations = stratus::migrate::load_migrations(&migrations_dir)
                    .expect("Failed to load migrations");

                stratus::migrate::print_migration_status(&migrations);
            }

            MigrateCommands::MigrateDiff {
                from: _,
                to,
                url: _,
                save: _,
                name: _,
            } => {
                println!("\n📐  Migrate Diff");
                println!("{}", "=".repeat(50));

                if let Some(schema_path) = to {
                    let schema_str =
                        fs::read_to_string(&schema_path).expect("Failed to read schema file");
                    let parsed_schema: stratus::schema::Schema =
                        serde_json::from_str(&schema_str).expect("Failed to parse schema");

                    println!("\nSchema: {}", schema_path.display());
                    println!("Tables: {}", parsed_schema.tables.len());

                    for (name, table) in &parsed_schema.tables {
                        println!("  + {}", name);
                        for col in table.columns.keys() {
                            println!("    - {}", col);
                        }
                    }

                    println!();
                    println!("TODO: Compare with database and generate SQL diff");
                    println!("Use --save to create migration file");
                } else {
                    println!("\nUsage:");
                    println!("  stratus migrate diff --from db --to schema.json");
                    println!("  stratus migrate diff --from schema_v1.json --to schema_v2.json");
                }
            }

            MigrateCommands::MigrateResolve {
                issue: _,
                migration: _,
            } => {
                println!("\n🔧  Migrate Resolve");
                println!("{}", "=".repeat(50));
                println!("Resolve migration issues like failed migrations.");
                println!();
                println!("TODO: Implement migration resolution");
            }
        },
    }
}
