use crate::models::{
    Author, AuthorName, CreateAuthorError, CreateAuthorRequest, DeleteAuthorError,
    DeleteAuthorRequest, EmailAddress, FindAllAuthorsError, FindAuthorError, FindAuthorRequest,
    UpdateAuthorError, UpdateAuthorRequest,
};
use crate::repositories::AuthorRepository;
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteRow};
use sqlx::{FromRow, Row, SqlitePool};
use std::str::FromStr;

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn establish_pool(path: &str) -> anyhow::Result<SqlitePool> {
    let opts = SqliteConnectOptions::from_str(path)
        .with_context(|| format!("Invalid database path {path}"))?
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);
    let pool = SqlitePool::connect_with(opts)
        .await
        .with_context(|| format!("Failed to open database at {path}"))?;

    MIGRATOR.run(&pool).await?;

    Ok(pool)
}

#[derive(Debug)]
pub struct DefaultAuthorRepository {
    pool: SqlitePool,
}

impl DefaultAuthorRepository {
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl<'r> FromRow<'r, SqliteRow> for Author {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let id = row.try_get("id")?;
        let name = row.try_get("name")?;
        let email = row.try_get("email")?;

        let name = AuthorName::new_unchecked(name);
        let email = EmailAddress::new_unchecked(email);
        Ok(Self::new(id, name, email))
    }
}

#[async_trait]
impl AuthorRepository for DefaultAuthorRepository {
    async fn create_author(&self, req: &CreateAuthorRequest) -> Result<Author, CreateAuthorError> {
        let author = sqlx::query_as("INSERT INTO author (name, email) VALUES (?, ?) RETURNING *")
            .bind(req.name().to_string())
            .bind(req.email().to_string())
            .fetch_one(&self.pool)
            .await
            .map_err(|err| {
                if is_unique_violation(&err) {
                    CreateAuthorError::Duplicate {
                        name: req.name().to_string(),
                    }
                } else {
                    let err = anyhow!(err).context(format!(
                        r#"Failed to create author with name "{}""#,
                        req.name()
                    ));
                    CreateAuthorError::Other(err)
                }
            })?;

        Ok(author)
    }

    async fn find_author(&self, req: &FindAuthorRequest) -> Result<Author, FindAuthorError> {
        let author = sqlx::query_as("SELECT id, name, email FROM author WHERE id = ?")
            .bind(req.id())
            .fetch_one(&self.pool)
            .await
            .map_err(|err| {
                if matches!(err, sqlx::Error::RowNotFound) {
                    FindAuthorError::NotFound { id: req.id() }
                } else {
                    let err = anyhow!(err).context(format!(
                        r#"Failed to retrieve author with id "{}""#,
                        req.id()
                    ));
                    FindAuthorError::Other(err)
                }
            })?;

        Ok(author)
    }

    async fn find_all_authors(&self) -> Result<Vec<Author>, FindAllAuthorsError> {
        let authors = sqlx::query_as("SELECT id, name, email FROM author")
            .fetch_all(&self.pool)
            .await
            .map_err(|err| {
                let err = anyhow!(err).context("Failed to retrieve all authors");
                FindAllAuthorsError(err)
            })?;

        Ok(authors)
    }

    async fn update_author(&self, req: &UpdateAuthorRequest) -> Result<(), UpdateAuthorError> {
        let mut parts = Vec::new();
        let mut binds = Vec::new();

        if let Some(name) = req.name() {
            parts.push("name = ?");
            binds.push(name.to_string());
        }
        if let Some(email) = req.email() {
            parts.push("email = ?");
            binds.push(email.to_string());
        }

        let query = format!("UPDATE author SET {} WHERE id = ?", parts.join(", "));
        let mut query = sqlx::query(&query);

        for bind in binds {
            query = query.bind(bind);
        }

        query
            .bind(req.id())
            .execute(&self.pool)
            .await
            .map_err(|err| {
                if matches!(err, sqlx::Error::RowNotFound) {
                    UpdateAuthorError::NotFound { id: req.id() }
                } else {
                    let err = anyhow!(err)
                        .context(format!(r#"Failed to update author with id "{}""#, req.id()));
                    UpdateAuthorError::Other(err)
                }
            })?;

        Ok(())
    }

    async fn delete_author(&self, req: &DeleteAuthorRequest) -> Result<(), DeleteAuthorError> {
        sqlx::query("DELETE FROM author WHERE id = ?")
            .bind(req.id())
            .execute(&self.pool)
            .await
            .map_err(|err| {
                if matches!(err, sqlx::Error::RowNotFound) {
                    DeleteAuthorError::NotFound { id: req.id() }
                } else {
                    let err = anyhow!(err)
                        .context(format!(r#"Failed to delete author with id "{}""#, req.id()));
                    DeleteAuthorError::Other(err)
                }
            })?;

        Ok(())
    }
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        return db_err.is_unique_violation();
    }

    false
}
