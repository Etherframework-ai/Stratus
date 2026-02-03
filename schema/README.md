# Stratus Schema JSON Schemas

This directory contains database-specific JSON Schema definitions for validating `schema.json` files.

## Database-Specific Schemas

| File | Database | Dialect |
|------|----------|---------|
| `postgresql.json` | PostgreSQL | `postgresql` |
| `mysql.json` | MySQL / MariaDB | `mysql` |
| `sqlite.json` | SQLite | `sqlite` |

## Usage

### VSCode Configuration

Copy `vscode-settings.json` to your project's `.vscode/settings.json`:

```json
{
  "json.schemas": [
    {
      "fileMatch": ["schema.json", "*.schema.json"],
      "url": "./node_modules/stratus/schema/postgresql.json",
      "name": "Stratus Schema"
    }
  ]
}
```

Or use pattern matching for different databases:

```json
{
  "json.schemas": [
    {
      "fileMatch": ["*postgres*.json"],
      "url": "./node_modules/stratus/schema/postgresql.json"
    },
    {
      "fileMatch": ["*mysql*.json", "*mariadb*.json"],
      "url": "./node_modules/stratus/schema/mysql.json"
    },
    {
      "fileMatch": ["*sqlite*.json"],
      "url": "./node_modules/stratus/schema/sqlite.json"
    }
  ]
}
```

### CLI Validation

Validate your schema with automatic dialect detection:

```bash
stratus validate --schema examples/schema.json
```

Or validate with explicit dialect:

```bash
# PostgreSQL
stratus validate --schema examples/schema.json --dialect postgresql

# MySQL
stratus validate --schema examples/mysql_schema.json --dialect mysql

# SQLite
stratus validate --schema examples/sqlite_schema.json --dialect sqlite
```

### Python Validation

```python
import json
from jsonschema import validate, Draft7Validator

# PostgreSQL
with open('schema/postgresql.json') as f:
    pg_schema = json.load(f)

with open('examples/schema.json') as f:
    data = json.load(f)

# Validate
validator = Draft7Validator(pg_schema)
errors = list(validator.iter_errors(data))
if errors:
    print(f"Validation errors: {errors}")
else:
    print("✓ Schema is valid!")
```

## Schema Structure by Database

### PostgreSQL Schema (`postgresql.json`)

PostgreSQL-specific features:
- Advanced types: `uuid`, `jsonb`, `timestamptz`, `inet`, `cidr`, `tsvector`, etc.
- IDENTITY columns (`GENERATED ALWAYS AS IDENTITY`)
- Generated columns (`GENERATED ALWAYS AS (...) STORED/VIRTUAL`)
- Advanced indexes: `btree`, `hash`, `gist`, `spgist`, `gin`, `brin`
- Partial indexes with `WHERE` clause
- Index storage parameters (`fillfactor`, `fastupdate`, etc.)
- Table partitioning: `RANGE`, `LIST`, `HASH`
- Table inheritance
- CHECK constraints
- EXCLUDE constraints
- Deferrable constraints
- Table options: `tablespace`, `fillfactor`, `toast_tuple_target`

**Example:**
```json
{
  "version": "1",
  "dialect": "postgresql",
  "tables": {
    "users": {
      "columns": {
        "id": {
          "name": "id",
          "type": "bigint",
          "isPrimaryKey": true,
          "identity": { "always": true }
        },
        "uuid": {
          "name": "uuid",
          "type": "uuid",
          "default": "gen_random_uuid()"
        },
        "tags": {
          "name": "tags",
          "type": "text",
          "arrayDimensions": 1
        }
      },
      "indexes": [
        {
          "name": "idx_users_email",
          "columns": ["email"],
          "unique": true,
          "method": "btree",
          "with": { "fillfactor": 90 }
        }
      ]
    }
  }
}
```

### MySQL Schema (`mysql.json`)

MySQL-specific features:
- Storage engines: `InnoDB`, `MyISAM`, `Memory`, `CSV`, `Archive`
- Character sets and collations: `utf8mb4`, `utf8mb4_unicode_ci`
- Auto-increment columns
- Column comments
- FULLTEXT indexes for text search
- SPATIAL indexes for geometry types
- ON UPDATE clause for timestamps
- Key block size
- Row format options

**Example:**
```json
{
  "version": "1",
  "dialect": "mysql",
  "tables": {
    "users": {
      "columns": {
        "id": {
          "name": "id",
          "type": "int",
          "unsigned": true,
          "autoIncrement": true,
          "isPrimaryKey": true
        },
        "email": {
          "name": "email",
          "type": "varchar",
          "size": 255,
          "collation": "utf8mb4_unicode_ci"
        }
      },
      "engine": "InnoDB",
      "charset": "utf8mb4",
      "collate": "utf8mb4_unicode_ci"
    }
  }
}
```

### SQLite Schema (`sqlite.json`)

SQLite-specific features:
- Flexible type affinity
- AUTOINCREMENT (limited to `INTEGER PRIMARY KEY`)
- Generated columns (`GENERATED ALWAYS AS`)
- WITHOUT ROWID optimization
- STRICT tables (type checking)
- Partial indexes with `WHERE` clause
- Collation sequences: `BINARY`, `NOCASE`, `RTRIM`, `UTF8`
- Deferred foreign keys

**Example:**
```json
{
  "version": "1",
  "dialect": "sqlite",
  "tables": {
    "users": {
      "columns": {
        "id": {
          "name": "id",
          "type": "integer",
          "isPrimaryKey": true,
          "autoIncrement": true
        },
        "name": {
          "name": "name",
          "type": "text",
          "collate": "NOCASE"
        }
      },
      "withoutRowid": false,
      "strict": true
    }
  }
}
```

## Type Mappings

### PostgreSQL → TypeScript

| PostgreSQL | TypeScript |
|------------|------------|
| `serial`, `bigint`, `int8` | `number` |
| `varchar`, `text`, `char` | `string` |
| `boolean` | `boolean` |
| `date`, `timestamp`, `timestamptz` | `Date` |
| `json`, `jsonb` | `unknown` |
| `uuid` | `string` |
| `bytea` | `Uint8Array` |
| `text[]` | `string[]` |

### MySQL → Python

| MySQL | Python |
|-------|--------|
| `int`, `bigint` | `int` |
| `varchar`, `text` | `str` |
| `bool` | `bool` |
| `datetime`, `timestamp` | `datetime.datetime` |
| `json` | `Any` |
| `blob` | `bytes` |

### SQLite → TypeScript

| SQLite | TypeScript |
|--------|------------|
| `integer` | `number` |
| `real` | `number` |
| `text` | `string` |
| `blob` | `Uint8Array` |
| `numeric` | `number` |

## Examples

See `examples/` directory for complete schema examples:
- `examples/schema.json` - PostgreSQL example
- `examples/schema_postgres.json` - Extended PostgreSQL example
- `examples/schema_mysql.json` - MySQL example (if available)
- `examples/schema_sqlite.json` - SQLite example (if available)
