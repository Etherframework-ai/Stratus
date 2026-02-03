use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct QueryFile {
    pub queries: Vec<Query>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub name: String,
    pub return_type: String,
    pub sql: String,
    pub params: Vec<Param>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub type_: String,
    pub ordinal: usize,
}

impl fmt::Display for QueryFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "QueryFile {{")?;
        for query in &self.queries {
            writeln!(f, "  {}", query)?;
        }
        write!(f, "}}")
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Query({}: {})", self.name, self.return_type)
    }
}
