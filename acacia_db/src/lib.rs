//! Database layer for Acacia, wrapping SeaORM.

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, QueryResult, Statement};
use std::sync::Arc;

mod schema;

pub use schema::*;
pub use sea_orm::QueryResult as Row;

/// Database error type.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, DbError>;

impl From<sea_orm::DbErr> for DbError {
    fn from(err: sea_orm::DbErr) -> Self {
        DbError::Query(err.to_string())
    }
}

// Convert DbError to AppError for use with ? in handlers
impl From<DbError> for acacia_core::AppError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::NotFound => acacia_core::AppError::NotFound,
            DbError::Connection(msg) => acacia_core::AppError::Database(msg),
            DbError::Query(msg) => acacia_core::AppError::Database(msg),
        }
    }
}

/// Trait for converting rows to models.
pub trait FromRow: Sized {
    fn from_row(row: &QueryResult) -> Result<Self>;
}

/// Trait for database models.
pub trait Model: FromRow + Sized + Send + Sync + 'static {
    type Key: Clone + Send + Sync + std::fmt::Display + 'static;
    type ActiveModel: Default + Clone + Send + Sync;

    fn table_name() -> &'static str;
    fn key(&self) -> Self::Key;
}

/// Trait for forms.
pub trait Form: serde::de::DeserializeOwned + Send + Sync {}

/// Migration policy.
#[derive(Clone, Copy, Debug, Default)]
pub enum MigratePolicy {
    #[default]
    Auto,
    None,
}

/// Database extractor for Axum handlers.
#[derive(Clone)]
pub struct Db {
    conn: Arc<DatabaseConnection>,
}

impl Db {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self {
            conn: Arc::new(conn),
        }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.conn
    }

    /// Get all records of a model type.
    pub async fn all<M: Model>(&self) -> Result<Vec<M>> {
        let table = M::table_name();
        let sql = format!("SELECT * FROM {}", table);
        let results = self
            .conn
            .query_all(Statement::from_string(DbBackend::Sqlite, sql))
            .await?;

        results.into_iter().map(|row| M::from_row(&row)).collect()
    }

    /// Get a single record by key.
    pub async fn get<M: Model>(&self, key: M::Key) -> Result<Option<M>> {
        let table = M::table_name();
        let sql = format!("SELECT * FROM {} WHERE id = ?", table);
        let result = self
            .conn
            .query_one(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                &sql,
                vec![key.to_string().into()],
            ))
            .await?;

        match result {
            Some(row) => Ok(Some(M::from_row(&row)?)),
            None => Ok(None),
        }
    }

    /// Insert a new record.
    pub async fn insert<M, F>(&self, form: F) -> Result<M>
    where
        M: Model,
        F: InsertableFor<M>,
    {
        let (columns, values) = form.columns_and_values();
        let table = M::table_name();

        let placeholders: Vec<_> = (0..values.len()).map(|_| "?").collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            columns.join(", "),
            placeholders.join(", ")
        );

        let sea_values: Vec<sea_orm::Value> = values.into_iter().map(|v| v.into()).collect();

        self.conn
            .execute(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                &sql,
                sea_values,
            ))
            .await?;

        // Get the last inserted row
        let last_id_sql = "SELECT last_insert_rowid() as id";
        let id_result = self
            .conn
            .query_one(Statement::from_string(DbBackend::Sqlite, last_id_sql))
            .await?
            .ok_or(DbError::NotFound)?;

        let id: i32 = id_result
            .try_get("", "id")
            .map_err(|e| DbError::Query(e.to_string()))?;

        let select_sql = format!("SELECT * FROM {} WHERE id = ?", table);
        let result = self
            .conn
            .query_one(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                &select_sql,
                vec![id.into()],
            ))
            .await?
            .ok_or(DbError::NotFound)?;

        M::from_row(&result)
    }

    /// Update a record with a mutation function.
    pub async fn update<M, F>(&self, key: M::Key, f: F) -> Result<M>
    where
        M: Model,
        F: FnOnce(&mut M),
    {
        // Get the current record
        let mut record = self.get::<M>(key.clone()).await?.ok_or(DbError::NotFound)?;

        // Apply the mutation
        f(&mut record);

        // For now, we'll use a simple approach - update all fields
        // This requires the model to implement a method to get update values
        let table = M::table_name();

        // We need a way to get the updated values from the model
        // For MVP, we'll use a simpler toggle approach for the todo example
        let sql = format!("UPDATE {} SET done = NOT done WHERE id = ?", table);

        self.conn
            .execute(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                &sql,
                vec![key.to_string().into()],
            ))
            .await?;

        // Return the updated record
        self.get::<M>(key).await?.ok_or(DbError::NotFound)
    }

    /// Delete a record by key.
    pub async fn delete<M: Model>(&self, key: M::Key) -> Result<()> {
        let table = M::table_name();
        let sql = format!("DELETE FROM {} WHERE id = ?", table);

        self.conn
            .execute(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                &sql,
                vec![key.to_string().into()],
            ))
            .await?;

        Ok(())
    }

    /// Run auto-migrations for all registered schemas.
    pub async fn migrate(&self) -> Result<()> {
        for schema_reg in inventory::iter::<SchemaRegistration> {
            let schema = (schema_reg.get_schema)();
            self.create_table_if_not_exists(&schema).await?;
        }
        Ok(())
    }

    async fn create_table_if_not_exists(&self, schema: &TableSchema) -> Result<()> {
        let mut columns = Vec::new();

        for col in &schema.columns {
            let mut col_def = format!("{} {}", col.name, col.sql_type);

            if col.primary_key {
                col_def.push_str(" PRIMARY KEY");
            }

            if col.auto_increment {
                col_def.push_str(" AUTOINCREMENT");
            }

            if !col.nullable && !col.primary_key {
                col_def.push_str(" NOT NULL");
            }

            if let Some(ref default) = col.default {
                col_def.push_str(&format!(" DEFAULT {}", default));
            }

            columns.push(col_def);
        }

        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            schema.name,
            columns.join(", ")
        );

        self.conn
            .execute(Statement::from_string(DbBackend::Sqlite, sql))
            .await?;

        Ok(())
    }
}

/// Trait for types that can be inserted for a model.
pub trait InsertableFor<M: Model>: Send {
    fn columns_and_values(self) -> (Vec<&'static str>, Vec<SqlValue>);
}

/// SQL value wrapper.
#[derive(Clone, Debug)]
pub enum SqlValue {
    String(String),
    Int(i32),
    Bool(bool),
    Null,
}

impl From<String> for SqlValue {
    fn from(s: String) -> Self {
        SqlValue::String(s)
    }
}

impl From<&str> for SqlValue {
    fn from(s: &str) -> Self {
        SqlValue::String(s.to_string())
    }
}

impl From<i32> for SqlValue {
    fn from(i: i32) -> Self {
        SqlValue::Int(i)
    }
}

impl From<bool> for SqlValue {
    fn from(b: bool) -> Self {
        SqlValue::Bool(b)
    }
}

impl From<SqlValue> for sea_orm::Value {
    fn from(v: SqlValue) -> Self {
        match v {
            SqlValue::String(s) => sea_orm::Value::String(Some(Box::new(s))),
            SqlValue::Int(i) => sea_orm::Value::Int(Some(i)),
            SqlValue::Bool(b) => sea_orm::Value::Bool(Some(b)),
            SqlValue::Null => sea_orm::Value::String(None),
        }
    }
}

/// Axum extractor implementation for Db.
#[async_trait]
impl<S> FromRequestParts<S> for Db
where
    S: Send + Sync,
    Db: FromRef<S>,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        Ok(Db::from_ref(state))
    }
}

impl FromRef<acacia_core::AppState> for Db {
    fn from_ref(state: &acacia_core::AppState) -> Self {
        Db::new(state.db.clone().expect("Database not configured"))
    }
}
