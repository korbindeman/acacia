//! Schema definitions for auto-migration.

/// Table schema definition.
#[derive(Clone, Debug)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnSchema>,
}

/// Column schema definition.
#[derive(Clone, Debug)]
pub struct ColumnSchema {
    pub name: String,
    pub sql_type: String,
    pub primary_key: bool,
    pub auto_increment: bool,
    pub nullable: bool,
    pub default: Option<String>,
}

/// Trait for models with schema.
pub trait HasSchema {
    fn schema() -> TableSchema;
}

/// Schema registration for inventory.
pub struct SchemaRegistration {
    pub get_schema: fn() -> TableSchema,
}

impl SchemaRegistration {
    pub const fn new(get_schema: fn() -> TableSchema) -> Self {
        Self { get_schema }
    }
}

inventory::collect!(SchemaRegistration);

/// Trait for mapping Rust types to SQL types.
pub trait SqlType {
    fn sql_type() -> String;
    fn default_value() -> Option<String> {
        None
    }
}

impl SqlType for i32 {
    fn sql_type() -> String {
        "INTEGER".to_string()
    }
}

impl SqlType for i64 {
    fn sql_type() -> String {
        "INTEGER".to_string()
    }
}

impl SqlType for String {
    fn sql_type() -> String {
        "TEXT".to_string()
    }
}

impl SqlType for bool {
    fn sql_type() -> String {
        "BOOLEAN".to_string()
    }
    fn default_value() -> Option<String> {
        Some("0".to_string())
    }
}

impl SqlType for f64 {
    fn sql_type() -> String {
        "REAL".to_string()
    }
}

impl<T: SqlType> SqlType for Option<T> {
    fn sql_type() -> String {
        T::sql_type()
    }
}
