use crate::ast::{Param, Query, QueryFile};
use std::str::Lines;

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t'
}

fn trim_ws(s: &str) -> &str {
    s.trim_start_matches(is_whitespace)
}

fn parse_identifier(s: &str) -> Option<(&str, String)> {
    let mut end = 0;
    for (i, c) in s.char_indices() {
        if c.is_alphanumeric() || c == '_' {
            end = i + c.len_utf8();
        } else {
            break;
        }
    }
    if end > 0 {
        Some((&s[end..], s[..end].to_string()))
    } else {
        None
    }
}

fn parse_name(line: &str) -> Option<(&str, String)> {
    let line = trim_ws(line);
    if !line.starts_with("name:") {
        return None;
    }
    let after = &line[5..];
    let after = trim_ws(after);
    parse_identifier(after)
}

fn parse_return_type(line: &str) -> Option<(&str, String)> {
    let line = trim_ws(line);
    if !line.starts_with(':') {
        return None;
    }
    let after = &line[1..];
    parse_identifier(trim_ws(after))
}

fn parse_param(line: &str) -> Option<(&str, (String, String))> {
    let line = trim_ws(line);
    let (rest, name) = parse_identifier(line)?;
    let rest = trim_ws(rest);
    if !rest.starts_with(':') {
        return None;
    }
    let after = &rest[1..];
    let (rest, type_) = parse_identifier(trim_ws(after))?;
    Some((rest, (name, type_)))
}

fn parse_query(lines: &mut Lines) -> Option<Query> {
    // Find header line
    let header_line = lines.next()?;
    let header_line = header_line.trim();

    // Skip empty lines
    if header_line.is_empty() {
        return parse_query(lines);
    }

    // Check for comment
    let header = if header_line.starts_with('#') {
        &header_line[1..]
    } else {
        header_line
    };

    // Parse name
    let (rest, name) = parse_name(header)?;
    let (rest, return_type) = parse_return_type(rest).unwrap_or((rest, "one".to_string()));

    // Parse params
    let mut params = Vec::new();
    let mut current = trim_ws(rest);
    while let Some((rest_after, (pname, ptype))) = parse_param(current) {
        params.push(Param {
            name: pname,
            type_: ptype,
            ordinal: params.len() + 1,
        });
        current = trim_ws(rest_after);
    }

    // Parse SQL lines
    let mut sql_parts = Vec::<String>::new();
    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        sql_parts.push(line.to_string());
    }

    let sql = sql_parts.join(" ");

    Some(Query {
        name,
        return_type,
        sql,
        params,
    })
}

pub fn parse(input: &str) -> Result<QueryFile, String> {
    let mut lines = input.lines();
    let mut queries = Vec::new();

    while let Some(query) = parse_query(&mut lines) {
        queries.push(query);
    }

    Ok(QueryFile { queries })
}

/// Represents a parsed SELECT column
#[derive(Debug, Clone)]
pub struct SelectColumn {
    pub table_name: Option<String>,
    pub column_name: String,
    pub is_wildcard: bool,
}

/// Extract tables from FROM clause
pub fn extract_tables_from_sql(sql: &str) -> Vec<String> {
    let mut tables = Vec::new();

    // Find FROM keyword
    if let Some(from_pos) = sql.to_lowercase().find("from") {
        let after_from = &sql[from_pos + 4..];

        // Find WHERE to limit our parsing
        let before_where = if let Some(where_pos) = after_from.to_lowercase().find("where") {
            &after_from[..where_pos]
        } else {
            after_from
        };

        // Trim and work with lowercase version
        let trimmed = before_where.trim();
        let lower_trimmed = trimmed.to_lowercase();

        let join_parts: Vec<&str> = if lower_trimmed.starts_with("join ") {
            // Edge case: starts with JOIN (no table before)
            vec!["", &trimmed[4..].trim_start()]
        } else if lower_trimmed.contains(" join ") {
            // Space before and after JOIN
            // Find position in lowercase, then use same position in original
            let pos = lower_trimmed.find(" join ").unwrap();
            let join_delim = &trimmed[pos..pos + 5]; // 5 = " join ".len()
            trimmed.split(join_delim).collect()
        } else if lower_trimmed.contains("join ") {
            // Space after JOIN (but no space before)
            let pos = lower_trimmed.find("join ").unwrap();
            let join_delim = &trimmed[pos..pos + 4]; // 4 = "join ".len()
            let parts: Vec<&str> = trimmed.split(join_delim).collect();
            if parts.len() >= 2 {
                vec![parts[0], parts[1]]
            } else {
                vec![trimmed]
            }
        } else {
            vec![trimmed]
        };

        for (i, part) in join_parts.iter().enumerate() {
            let part = part.trim();

            if part.is_empty() {
                continue;
            }

            if i == 0 {
                // First part is after FROM, before first JOIN
                // Get the first word as table name
                let table_name: String = part
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !table_name.is_empty() {
                    tables.push(table_name);
                }
            } else {
                // Parts after JOIN
                // Skip join type keywords like "INNER", "LEFT", etc.
                let mut remaining = part;
                loop {
                    let next_word: String = remaining
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();

                    if next_word.is_empty() {
                        break;
                    }

                    // Skip join type keywords
                    if next_word == "inner"
                        || next_word == "left"
                        || next_word == "right"
                        || next_word == "outer"
                        || next_word == "cross"
                        || next_word == "full"
                    {
                        remaining = remaining[next_word.len()..].trim_start();
                        continue;
                    }

                    // This should be a table name
                    tables.push(next_word);
                    break;
                }
            }
        }
    }

    tables
}

/// Extract SELECT columns from SQL query
pub fn extract_select_columns(sql: &str) -> Vec<SelectColumn> {
    let mut columns = Vec::new();

    // Find SELECT keyword
    if let Some(select_pos) = sql.to_lowercase().find("select") {
        let after_select = &sql[select_pos + 6..];

        // Find FROM keyword to get end of SELECT clause
        let from_pos = after_select.to_lowercase().find("from");
        let select_content = if let Some(pos) = from_pos {
            &after_select[..pos]
        } else {
            after_select
        };

        // Split by comma
        let parts: Vec<&str> = select_content.split(',').collect();

        for part in parts {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            // Check for wildcard
            if part == "*" {
                columns.push(SelectColumn {
                    table_name: None,
                    column_name: "*".to_string(),
                    is_wildcard: true,
                });
                continue;
            }

            // Check for table.*
            if part.ends_with(".*") {
                let table_name = &part[..part.len() - 2];
                columns.push(SelectColumn {
                    table_name: Some(table_name.to_string()),
                    column_name: "*".to_string(),
                    is_wildcard: true,
                });
                continue;
            }

            // Check for table.column
            if let Some(dot_pos) = part.find('.') {
                let table_name = &part[..dot_pos].trim();
                let col_name = &part[dot_pos + 1..].trim();
                columns.push(SelectColumn {
                    table_name: Some(table_name.to_string()),
                    column_name: col_name.to_string(),
                    is_wildcard: false,
                });
            } else {
                columns.push(SelectColumn {
                    table_name: None,
                    column_name: part.to_string(),
                    is_wildcard: false,
                });
            }
        }
    }

    columns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_query() {
        let input = "# name: GetUser :one\nSELECT * FROM users WHERE id = $1;\n";
        let result = parse(input);
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        let qf = result.unwrap();
        assert_eq!(qf.queries.len(), 1);
        let q = &qf.queries[0];
        assert_eq!(q.name, "GetUser");
        assert_eq!(q.return_type, "one");
        assert_eq!(q.sql, "SELECT * FROM users WHERE id = $1;");
    }

    #[test]
    fn test_parse_multiple_queries() {
        let input = "# name: GetUser :one\nSELECT * FROM users WHERE id = $1;\n\n# name: ListUsers :many\nSELECT * FROM users;\n";
        let result = parse(input);
        assert!(result.is_ok());
        let qf = result.unwrap();
        assert_eq!(qf.queries.len(), 2);
    }

    #[test]
    fn test_parse_params() {
        let input = "# name: GetUserById :one id: number\nSELECT * FROM users WHERE id = $1;\n";
        let result = parse(input);
        assert!(result.is_ok());
        let qf = result.unwrap();
        let q = &qf.queries[0];
        assert_eq!(q.params.len(), 1);
        assert_eq!(q.params[0].name, "id");
        assert_eq!(q.params[0].type_, "number");
    }
}
