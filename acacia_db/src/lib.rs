//! Database layer for Acacia, wrapping SeaORM 2.0 with entity-first workflow.

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, ModelTrait, PrimaryKeyTrait, Schema,
};
use std::sync::Arc;

// Re-export SeaORM types that users need
pub use sea_orm::{
    ActiveValue, ColumnTrait, DeriveEntityModel, DeriveRelation, EntityName, EnumIter,
    IntoActiveModel as _, QueryFilter, Set,
};

/// Registration for entity schema creation.
/// Each entity registers itself so migrations can create the table.
pub struct EntityRegistration {
    pub create_table: fn(&Schema) -> sea_orm::sea_query::TableCreateStatement,
}

impl EntityRegistration {
    pub const fn new(
        create_table: fn(&Schema) -> sea_orm::sea_query::TableCreateStatement,
    ) -> Self {
        Self { create_table }
    }
}

inventory::collect!(EntityRegistration);

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
        match err {
            sea_orm::DbErr::RecordNotFound(_) => DbError::NotFound,
            _ => DbError::Query(err.to_string()),
        }
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

/// Migration policy.
#[derive(Clone, Copy, Debug, Default)]
pub enum MigratePolicy {
    #[default]
    Auto,
    None,
}

/// Database handle for Axum handlers.
///
/// This wraps a SeaORM DatabaseConnection and provides convenient methods
/// for common database operations.
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

    /// Get the underlying SeaORM connection for advanced operations.
    pub fn connection(&self) -> &DatabaseConnection {
        &self.conn
    }

    /// Get all records of a model type.
    ///
    /// # Example
    /// ```ignore
    /// let tasks = db.all::<Task>().await?;
    /// ```
    pub async fn all<M>(&self) -> Result<Vec<M>>
    where
        M: ModelTrait,
        M::Entity: EntityTrait<Model = M>,
    {
        M::Entity::find().all(&*self.conn).await.map_err(Into::into)
    }

    /// Get a single record by primary key.
    ///
    /// # Example
    /// ```ignore
    /// let task = db.get::<Task>(1).await?;
    /// ```
    pub async fn get<M>(
        &self,
        id: <<M::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Result<Option<M>>
    where
        M: ModelTrait,
        M::Entity: EntityTrait<Model = M>,
    {
        M::Entity::find_by_id(id)
            .one(&*self.conn)
            .await
            .map_err(Into::into)
    }

    /// Insert a new record from a form/DTO.
    ///
    /// # Example
    /// ```ignore
    /// let task = db.insert::<Task, _>(NewTask { title: "...".into() }).await?;
    /// ```
    pub async fn insert<M, F>(&self, form: F) -> Result<M>
    where
        M: ModelTrait + IntoActiveModel<<M::Entity as EntityTrait>::ActiveModel>,
        M::Entity: EntityTrait<Model = M>,
        F: IntoActiveModel<<M::Entity as EntityTrait>::ActiveModel>,
        <M::Entity as EntityTrait>::ActiveModel: ActiveModelTrait<Entity = M::Entity> + Send,
    {
        let active_model = form.into_active_model();
        let result = active_model.insert(&*self.conn).await?;
        Ok(result)
    }

    /// Update a record by applying a mutation function to the model.
    ///
    /// The closure receives a mutable reference to the model data,
    /// which you can modify directly. The changes are then saved to the database.
    ///
    /// # Example
    /// ```ignore
    /// let task = db.update::<Task>(1, |task| {
    ///     task.done = !task.done;
    /// }).await?;
    /// ```
    pub async fn update<M, F>(
        &self,
        id: <<M::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
        mutate: F,
    ) -> Result<M>
    where
        M: ModelTrait + IntoActiveModel<<M::Entity as EntityTrait>::ActiveModel>,
        M::Entity: EntityTrait<Model = M>,
        <M::Entity as EntityTrait>::ActiveModel: ActiveModelTrait<Entity = M::Entity> + Send,
        F: FnOnce(&mut M),
    {
        let mut model = self.get::<M>(id).await?.ok_or(DbError::NotFound)?;

        // Apply the user's mutation to the model
        mutate(&mut model);

        // Convert to ActiveModel and save
        let active_model = model.into_active_model();
        let updated = active_model.update(&*self.conn).await?;
        Ok(updated)
    }

    /// Toggle a boolean field on a record.
    ///
    /// # Example
    /// ```ignore
    /// let task = db.toggle::<Task>(id, |t| &mut t.done).await?;
    /// ```
    pub async fn toggle<M, F>(
        &self,
        id: <<M::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
        field: F,
    ) -> Result<M>
    where
        M: ModelTrait + IntoActiveModel<<M::Entity as EntityTrait>::ActiveModel>,
        M::Entity: EntityTrait<Model = M>,
        <M::Entity as EntityTrait>::ActiveModel: ActiveModelTrait<Entity = M::Entity> + Send,
        F: FnOnce(&mut M) -> &mut bool,
    {
        self.update::<M, _>(id, |model| {
            let field = field(model);
            *field = !*field;
        })
        .await
    }

    /// Delete a record by primary key.
    ///
    /// # Example
    /// ```ignore
    /// db.delete::<Task>(1).await?;
    /// ```
    pub async fn delete<M>(
        &self,
        id: <<M::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Result<()>
    where
        M: ModelTrait + IntoActiveModel<<M::Entity as EntityTrait>::ActiveModel>,
        M::Entity: EntityTrait<Model = M>,
        <M::Entity as EntityTrait>::ActiveModel:
            ActiveModelTrait<Entity = M::Entity> + ActiveModelBehavior + Send,
    {
        let model = self.get::<M>(id).await?.ok_or(DbError::NotFound)?;

        model.delete(&*self.conn).await?;
        Ok(())
    }

    /// Run schema synchronization for all registered entities.
    ///
    /// This creates tables for all entities that have been registered
    /// via the #[model] attribute macro.
    pub async fn migrate(&self) -> Result<()> {
        let backend = self.conn.get_database_backend();
        let schema = Schema::new(backend);

        for registration in inventory::iter::<EntityRegistration> {
            let stmt = (registration.create_table)(&schema);
            // Use IF NOT EXISTS for idempotent migrations
            self.conn
                .execute(&stmt)
                .await
                .map_err(|e| DbError::Query(e.to_string()))?;
        }
        Ok(())
    }
}

/// Trait for forms that can be converted to an ActiveModel for insertion.
pub trait Form: serde::de::DeserializeOwned + Send + Sync {}

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
