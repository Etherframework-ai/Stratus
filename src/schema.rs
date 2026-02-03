use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Schema {
    pub version: Option<String>,
    pub dialect: Option<String>,
    pub tables: HashMap<String, Table>,
    pub enums: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Table {
    pub columns: HashMap<String, Column>,
    pub indexes: Option<Vec<Index>>,
    pub constraints: Option<Vec<TableConstraint>>,
    #[serde(default)]
    pub options: TableOptions,
    #[serde(default)]
    pub partitions: Vec<Partition>,
    #[serde(default)]
    pub inherits: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Column {
    #[serde(rename = "name")]
    pub column_name: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub size: Option<usize>,
    #[serde(default)]
    #[serde(rename = "arrayDimensions")]
    pub array_dimensions: Option<usize>,
    #[serde(default)]
    #[serde(rename = "isPrimaryKey")]
    pub is_primary_key: bool,
    #[serde(default)]
    #[serde(rename = "isNotNull")]
    pub is_not_null: bool,
    #[serde(default)]
    #[serde(rename = "isUnique")]
    pub is_unique: bool,
    #[serde(default)]
    pub default: Option<String>,
    pub identity: Option<Identity>,
    pub generated: Option<GeneratedAs>,
    #[serde(default)]
    pub collation: Option<String>,
    #[serde(default)]
    pub storage: Option<StorageType>,
    #[serde(default)]
    pub statistics: Option<i32>,
    #[serde(default)]
    pub attributes: ColumnAttributes,
    #[serde(default)]
    pub references: Option<ForeignKey>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ColumnAttributes {
    #[serde(default)]
    pub is_identity: bool,
    #[serde(default)]
    pub is_generated: bool,
    #[serde(default)]
    pub is_computed: bool,
    #[serde(default)]
    pub compression: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Identity {
    pub sequence: Option<SequenceOptions>,
    #[serde(default)]
    pub always: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeneratedAs {
    #[serde(default)]
    pub always: bool,
    pub expression: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SequenceOptions {
    #[serde(default)]
    pub start: Option<i64>,
    #[serde(default)]
    pub minvalue: Option<i64>,
    #[serde(default)]
    pub maxvalue: Option<i64>,
    #[serde(default)]
    pub increment: Option<i64>,
    #[serde(default)]
    pub cycle: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TableOptions {
    pub tablespace: Option<String>,
    pub fillfactor: Option<u32>,
    #[serde(default)]
    pub toast_tuple_target: Option<u32>,
    #[serde(default)]
    pub autovacuum_enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    #[serde(default)]
    pub unique: bool,
    #[serde(default)]
    pub if_not_exists: bool,
    pub method: Option<IndexMethod>,
    pub tablespace: Option<String>,
    pub with: Option<IndexWithOptions>,
    pub where_clause: Option<String>,
    pub nulls_not_distinct: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TableConstraint {
    pub name: Option<String>,
    #[serde(rename = "constraintType")]
    pub constraint_type: ConstraintType,
    #[serde(default)]
    pub columns: Vec<String>,
    pub expression: Option<String>,
    pub references: Option<ForeignKey>,
    #[serde(default)]
    pub deferrable: bool,
    #[serde(default)]
    pub initially_deferred: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForeignKey {
    pub table: String,
    pub column: String,
    #[serde(default)]
    pub on_delete: Option<OnDeleteAction>,
    #[serde(default)]
    pub on_update: Option<OnUpdateAction>,
    #[serde(default)]
    pub match_type: Option<MatchType>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Partition {
    pub name: String,
    pub partition_type: PartitionType,
    pub key: Vec<String>,
    pub range_from: Option<Vec<String>>,
    pub range_to: Option<Vec<String>>,
    pub values: Option<Vec<String>>,
    pub tablespace: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum IndexMethod {
    #[serde(rename = "btree")]
    BTree,
    #[serde(rename = "hash")]
    Hash,
    #[serde(rename = "gist")]
    GiST,
    #[serde(rename = "spgist")]
    SPGiST,
    #[serde(rename = "gin")]
    GIN,
    #[serde(rename = "brin")]
    BRIN,
    #[serde(other)]
    Other,
}

impl Default for IndexMethod {
    fn default() -> Self {
        IndexMethod::BTree
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum ConstraintType {
    #[serde(rename = "primary key")]
    PrimaryKey,
    #[serde(rename = "unique")]
    Unique,
    #[serde(rename = "check")]
    Check,
    #[serde(rename = "exclude")]
    Exclude,
    #[serde(rename = "foreign key")]
    ForeignKey,
}

#[derive(Debug, Clone, Deserialize)]
pub enum OnDeleteAction {
    #[serde(rename = "cascade")]
    Cascade,
    #[serde(rename = "setNull")]
    SetNull,
    #[serde(rename = "setDefault")]
    SetDefault,
    #[serde(rename = "restrict")]
    Restrict,
    #[serde(rename = "noAction")]
    NoAction,
    #[serde(other)]
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub enum OnUpdateAction {
    #[serde(rename = "cascade")]
    Cascade,
    #[serde(rename = "setNull")]
    SetNull,
    #[serde(rename = "setDefault")]
    SetDefault,
    #[serde(rename = "restrict")]
    Restrict,
    #[serde(rename = "noAction")]
    NoAction,
    #[serde(other)]
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub enum MatchType {
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "partial")]
    Partial,
    #[serde(rename = "simple")]
    Simple,
}

#[derive(Debug, Clone, Deserialize)]
pub enum PartitionType {
    #[serde(rename = "range")]
    Range,
    #[serde(rename = "list")]
    List,
    #[serde(rename = "hash")]
    Hash,
}

#[derive(Debug, Clone, Deserialize)]
pub enum StorageType {
    #[serde(rename = "plain")]
    Plain,
    #[serde(rename = "external")]
    External,
    #[serde(rename = "extended")]
    Extended,
    #[serde(rename = "main")]
    Main,
}

impl Default for OnDeleteAction {
    fn default() -> Self {
        Self::None
    }
}

impl Default for OnUpdateAction {
    fn default() -> Self {
        Self::None
    }
}

impl Column {
    pub fn is_primary_key(&self) -> bool {
        self.is_primary_key
    }

    pub fn is_not_null(&self) -> bool {
        self.is_not_null
    }

    pub fn is_unique(&self) -> bool {
        self.is_unique
    }

    pub fn get_sql_type(&self) -> String {
        let base = if let Some(size) = self.size {
            format!("{}({})", self.data_type, size)
        } else {
            self.data_type.clone()
        };
        if let Some(dims) = self.array_dimensions {
            return format!("{}{}", base, "[]".repeat(dims));
        }
        base
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndexWithOptions {
    pub fillfactor: Option<u32>,
    pub deduplicate_items: Option<bool>,
    pub buffering: Option<bool>,
    pub fastupdate: Option<bool>,
    pub pages_per_range: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_extended_schema() {
        let json = r#"{
          "version": "1",
          "dialect": "postgresql",
          "tables": {
            "users": {
              "columns": {
                "id": {
                  "name": "id",
                  "type": "bigint",
                  "isPrimaryKey": true,
                  "isNotNull": true,
                  "identity": {
                    "always": true
                  }
                },
                "email": {
                  "name": "email",
                  "type": "varchar",
                  "size": 255,
                  "isNotNull": true,
                  "isUnique": true,
                  "collation": "en_US.utf8"
                },
                "tags": {
                  "name": "tags",
                  "type": "text",
                  "arrayDimensions": 1
                },
                "settings": {
                  "name": "settings",
                  "type": "jsonb"
                }
              },
              "indexes": [
                {
                  "name": "idx_users_email",
                  "columns": ["email"],
                  "unique": true,
                  "method": "btree"
                }
              ],
              "constraints": [
                {
                  "name": "chk_users_email_format",
                  "constraintType": "check",
                  "expression": "email ~ '^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$'"
                }
              ],
              "options": {
                "fillfactor": 90
              }
            }
          }
        }"#;

        let schema: Schema = serde_json::from_str(json).expect("Failed to parse");
        assert_eq!(schema.tables.len(), 1);

        let users = &schema.tables["users"];
        let email = users.columns.get("email").unwrap();
        assert!(email.is_unique);
        assert_eq!(email.collation, Some("en_US.utf8".to_string()));

        let tags = users.columns.get("tags").unwrap();
        assert_eq!(tags.array_dimensions, Some(1));
    }
}
