use crate::model::{Author, AuthorName, CreateAuthorError, CreateAuthorRequest, EmailAddress};
use crate::store::AuthorRepository;
use anyhow::{anyhow, Context};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteRow};
use sqlx::{FromRow, Row, SqlitePool};
use std::str::FromStr;

const UNIQUE_CONSTRAINT_VIOLATION_CODE: &str = "2067";

#[derive(Debug, Clone)]
pub struct Sqlite {
    pool: SqlitePool,
}

impl Sqlite {
    pub async fn new(path: &str) -> anyhow::Result<Self> {
        let opts = SqliteConnectOptions::from_str(path)
            .with_context(|| format!("Invalid database path {path}"))?
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal);
        let pool = SqlitePool::connect_with(opts)
            .await
            .with_context(|| format!("Failed to open database at {path}"))?;

        Ok(Self { pool })
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

impl AuthorRepository for Sqlite {
    async fn create_author(&self, req: &CreateAuthorRequest) -> Result<Author, CreateAuthorError> {
        let author = sqlx::query_as("INSERT INTO author (name, email) VALUES (?, ?) RETURNING *")
            .bind(req.name().to_string())
            .bind(req.email().to_string())
            .fetch_one(&self.pool)
            .await
            .map_err(|err| {
                if is_unique_constraint_violation(&err) {
                    CreateAuthorError::Duplicate {
                        name: req.name().to_string(),
                    }
                } else {
                    let err = anyhow!(err).context(format!(
                        "Failed to create author with name \"{}\"",
                        req.name()
                    ));
                    CreateAuthorError::Unknown(err)
                }
            })?;

        Ok(author)
    }
}

fn is_unique_constraint_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(ref db_err) = err {
        if let Some(code) = db_err.code() {
            return UNIQUE_CONSTRAINT_VIOLATION_CODE == code;
        }
    }

    false
}
