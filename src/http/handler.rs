use crate::http::AppState;
use crate::model::{
    Author, AuthorName, AuthorNameEmptyError, CreateAuthorError, CreateAuthorRequest,
    DeleteAuthorError, DeleteAuthorRequest, EmailAddress, EmailAddressError, FindAllAuthorsError,
    FindAuthorError, FindAuthorRequest,
};
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct HttpSuccess<T>(StatusCode, T);

impl<T: Serialize> HttpSuccess<T> {
    pub const fn new(status: StatusCode, data: T) -> Self {
        Self(status, data)
    }
}

impl<T: Serialize> IntoResponse for HttpSuccess<T> {
    fn into_response(self) -> axum::response::Response {
        (self.0, Json(self.1)).into_response()
    }
}

#[derive(Error, Debug)]
#[error("{1}")]
pub struct HttpError(StatusCode, String);

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        (self.0, Json(self.1)).into_response()
    }
}

impl From<ParseCreateAuthorHttpRequestError> for HttpError {
    fn from(err: ParseCreateAuthorHttpRequestError) -> Self {
        let msg = err.to_string();
        Self(StatusCode::UNPROCESSABLE_ENTITY, msg)
    }
}

impl From<CreateAuthorError> for HttpError {
    fn from(err: CreateAuthorError) -> Self {
        match err {
            CreateAuthorError::Duplicate { name } => Self(
                StatusCode::CONFLICT,
                format!(r#"author with name "{name}" already exists"#),
            ),
            CreateAuthorError::Other(cause) => {
                tracing::error!("{cause:?}\n{}", cause.backtrace());
                Self(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        }
    }
}

impl From<FindAuthorError> for HttpError {
    fn from(err: FindAuthorError) -> Self {
        match err {
            FindAuthorError::NotFound { id } => Self(
                StatusCode::NOT_FOUND,
                format!(r#"author with id "{id}" does not exist"#),
            ),
            FindAuthorError::Other(cause) => {
                tracing::error!("{cause:?}\n{}", cause.backtrace());
                Self(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        }
    }
}

impl From<FindAllAuthorsError> for HttpError {
    fn from(err: FindAllAuthorsError) -> Self {
        match err {
            FindAllAuthorsError(cause) => {
                tracing::error!("{cause:?}\n{}", cause.backtrace());
                Self(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        }
    }
}

impl From<DeleteAuthorError> for HttpError {
    fn from(err: DeleteAuthorError) -> Self {
        match err {
            DeleteAuthorError::NotFound { id } => Self(
                StatusCode::NOT_FOUND,
                format!(r#"author with id "{id}" does not exist"#),
            ),
            DeleteAuthorError::Other(cause) => {
                tracing::error!("{cause:?}\n{}", cause.backtrace());
                Self(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        }
    }
}

impl From<ParseIdError> for HttpError {
    fn from(err: ParseIdError) -> Self {
        Self(
            StatusCode::BAD_REQUEST,
            format!(r#"Cannot parse id from "{}""#, err.id),
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateAuthorHttpRequest {
    name: String,
    email: String,
}

#[derive(Error, Debug)]
#[error(transparent)]
pub enum ParseCreateAuthorHttpRequestError {
    Name(#[from] AuthorNameEmptyError),
    Email(#[from] EmailAddressError),
}

impl TryFrom<CreateAuthorHttpRequest> for CreateAuthorRequest {
    type Error = ParseCreateAuthorHttpRequestError;

    fn try_from(value: CreateAuthorHttpRequest) -> Result<Self, Self::Error> {
        let name = AuthorName::new(&value.name)?;
        let email = EmailAddress::new(&value.email)?;
        Ok(Self::new(name, email))
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct CreateAuthorHttpResponse {
    id: u64,
}

#[derive(Debug)]
pub struct ParseIdError {
    id: String,
}

impl std::fmt::Display for ParseIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"Cannot parse id from "{}""#, self.id)
    }
}

impl From<Author> for CreateAuthorHttpResponse {
    fn from(value: Author) -> Self {
        Self { id: value.id() }
    }
}

impl TryFrom<String> for FindAuthorRequest {
    type Error = ParseIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let id = value
            .parse::<u64>()
            .map_err(|_| ParseIdError { id: value })?;
        Ok(Self::new(id))
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct FindAuthorHttpResponse {
    id: u64,
    name: String,
    email: String,
}

impl From<Author> for FindAuthorHttpResponse {
    fn from(value: Author) -> Self {
        Self {
            id: value.id(),
            name: value.name().to_string(),
            email: value.email().to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct FindAllAuthorsHttpResponse(Vec<FindAuthorHttpResponse>);

impl From<Vec<Author>> for FindAllAuthorsHttpResponse {
    fn from(values: Vec<Author>) -> Self {
        let vec = values
            .into_iter()
            .map(FindAuthorHttpResponse::from)
            .collect();
        Self(vec)
    }
}

impl TryFrom<String> for DeleteAuthorRequest {
    type Error = ParseIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let id = value
            .parse::<u64>()
            .map_err(|_| ParseIdError { id: value })?;
        Ok(Self::new(id))
    }
}

pub async fn create_author(
    State(state): State<AppState>,
    Json(body): Json<CreateAuthorHttpRequest>,
) -> Result<HttpSuccess<CreateAuthorHttpResponse>, HttpError> {
    let req = body.try_into()?;
    state
        .author_repo
        .create_author(&req)
        .await
        .map_err(HttpError::from)
        .map(|author| HttpSuccess::new(StatusCode::CREATED, author.into()))
}

pub async fn find_author(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<HttpSuccess<FindAuthorHttpResponse>, HttpError> {
    let req = id.try_into()?;
    state
        .author_repo
        .find_author(&req)
        .await
        .map_err(HttpError::from)
        .map(|author| HttpSuccess::new(StatusCode::OK, author.into()))
}

pub async fn find_all_authors(
    State(state): State<AppState>,
) -> Result<HttpSuccess<FindAllAuthorsHttpResponse>, HttpError> {
    state
        .author_repo
        .find_all_authors()
        .await
        .map_err(HttpError::from)
        .map(|authors| HttpSuccess::new(StatusCode::OK, authors.into()))
}

pub async fn delete_author(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<HttpSuccess<()>, HttpError> {
    let req = id.try_into()?;
    state
        .author_repo
        .delete_author(&req)
        .await
        .map_err(HttpError::from)
        .map(|()| HttpSuccess::new(StatusCode::NO_CONTENT, ()))
}

#[cfg(test)]
mod tests {
    use crate::http::AppState;
    use crate::http::handler::{
        CreateAuthorHttpRequest, CreateAuthorHttpResponse, FindAllAuthorsHttpResponse,
        FindAuthorHttpResponse, HttpSuccess, create_author, delete_author, find_all_authors,
        find_author,
    };
    use crate::model::{
        Author, AuthorName, CreateAuthorError, CreateAuthorRequest, DeleteAuthorError,
        DeleteAuthorRequest, EmailAddress, FindAllAuthorsError, FindAuthorError, FindAuthorRequest,
    };
    use crate::store::AuthorRepository;
    use anyhow::anyhow;
    use async_trait::async_trait;
    use axum::Json;
    use axum::extract::{Path, State};
    use axum::http::StatusCode;
    use std::mem;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct MockAuthorRepository {
        create: Arc<Mutex<Result<Author, CreateAuthorError>>>,
        find: Arc<Mutex<Result<Author, FindAuthorError>>>,
        find_all: Arc<Mutex<Result<Vec<Author>, FindAllAuthorsError>>>,
        delete: Arc<Mutex<Result<(), DeleteAuthorError>>>,
    }

    impl MockAuthorRepository {
        pub fn new() -> Self {
            Self {
                create: Arc::new(Mutex::new(Err(CreateAuthorError::Other(anyhow!(
                    "substitute error"
                ))))),
                find: Arc::new(Mutex::new(Err(FindAuthorError::Other(anyhow!(
                    "substitute error"
                ))))),
                find_all: Arc::new(Mutex::new(Err(FindAllAuthorsError(anyhow!(
                    "substitute error"
                ))))),
                delete: Arc::new(Mutex::new(Err(DeleteAuthorError::Other(anyhow!(
                    "substitute error"
                ))))),
            }
        }
    }

    #[async_trait]
    impl AuthorRepository for MockAuthorRepository {
        async fn create_author(
            &self,
            _: &CreateAuthorRequest,
        ) -> Result<Author, CreateAuthorError> {
            let mut guard = self.create.lock();
            let mut result = Err(CreateAuthorError::Other(anyhow!("substitute error")));
            mem::swap(guard.as_deref_mut().unwrap(), &mut result);
            result
        }

        async fn find_author(&self, _: &FindAuthorRequest) -> Result<Author, FindAuthorError> {
            let mut guard = self.find.lock();
            let mut result = Err(FindAuthorError::Other(anyhow!("substitute error")));
            mem::swap(guard.as_deref_mut().unwrap(), &mut result);
            result
        }

        async fn find_all_authors(&self) -> Result<Vec<Author>, FindAllAuthorsError> {
            let mut guard = self.find_all.lock();
            let mut result = Err(FindAllAuthorsError(anyhow!("substitute error")));
            mem::swap(guard.as_deref_mut().unwrap(), &mut result);
            result
        }

        async fn delete_author(&self, _: &DeleteAuthorRequest) -> Result<(), DeleteAuthorError> {
            let mut guard = self.delete.lock();
            let mut result = Err(DeleteAuthorError::Other(anyhow!("substitute error")));
            mem::swap(guard.as_deref_mut().unwrap(), &mut result);
            result
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_author_handler_success() {
        let author_id = 1;
        let author_name = AuthorName::new("JRR Tolkien").unwrap();
        let author_email = EmailAddress::new("jrr.tolkien@example.com").unwrap();
        let repo = MockAuthorRepository {
            create: Arc::new(Mutex::new(Ok(Author::new(
                author_id,
                author_name.clone(),
                author_email.clone(),
            )))),
            ..MockAuthorRepository::new()
        };
        let state = State(AppState::new(repo));
        let body = Json(CreateAuthorHttpRequest {
            name: author_name.to_string(),
            email: author_email.to_string(),
        });
        let expected = HttpSuccess::new(
            StatusCode::CREATED,
            CreateAuthorHttpResponse { id: author_id },
        );
        let actual = create_author(state, body).await;
        assert!(
            actual.is_ok(),
            "expected create author to succeed, but got {actual:?}",
        );
        let actual = actual.unwrap();
        assert_eq!(
            expected, actual,
            "expected ApiSuccess {expected:?}, but got {actual:?}",
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn find_author_handler_success() {
        let author_id = 1;
        let author_name = AuthorName::new("JRR Tolkien").unwrap();
        let author_email = EmailAddress::new("jrr.tolkien@example.com").unwrap();
        let repo = MockAuthorRepository {
            find: Arc::new(Mutex::new(Ok(Author::new(
                author_id,
                author_name.clone(),
                author_email.clone(),
            )))),
            ..MockAuthorRepository::new()
        };
        let path = Path(author_id.to_string());
        let state = State(AppState::new(repo));
        let expected = HttpSuccess::new(
            StatusCode::OK,
            FindAuthorHttpResponse {
                id: author_id,
                name: author_name.to_string(),
                email: author_email.to_string(),
            },
        );
        let actual = find_author(path, state).await;
        assert!(
            actual.is_ok(),
            "expected find author to succeed, but got {actual:?}",
        );
        let actual = actual.unwrap();
        assert_eq!(
            expected, actual,
            "expected ApiSuccess {expected:?}, but got {actual:?}",
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn find_all_authors_handler_success() {
        let author_id = 1;
        let author_name = AuthorName::new("JRR Tolkien").unwrap();
        let author_email = EmailAddress::new("jrr.tolkien@example.com").unwrap();
        let repo = MockAuthorRepository {
            find_all: Arc::new(Mutex::new(Ok(vec![Author::new(
                author_id,
                author_name.clone(),
                author_email.clone(),
            )]))),
            ..MockAuthorRepository::new()
        };
        let state = State(AppState::new(repo));
        let expected = HttpSuccess::new(
            StatusCode::OK,
            FindAllAuthorsHttpResponse(vec![FindAuthorHttpResponse {
                id: author_id,
                name: author_name.to_string(),
                email: author_email.to_string(),
            }]),
        );
        let actual = find_all_authors(state).await;
        assert!(
            actual.is_ok(),
            "expected find author to succeed, but got {actual:?}",
        );
        let actual = actual.unwrap();
        assert_eq!(
            expected, actual,
            "expected ApiSuccess {expected:?}, but got {actual:?}",
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn delete_author_handler_success() {
        let author_id = 1;
        let repo = MockAuthorRepository {
            delete: Arc::new(Mutex::new(Ok(()))),
            ..MockAuthorRepository::new()
        };
        let path = Path(author_id.to_string());
        let state = State(AppState::new(repo));
        let expected = HttpSuccess::new(StatusCode::NO_CONTENT, ());
        let actual = delete_author(path, state).await;
        assert!(
            actual.is_ok(),
            "expected delete author to succeed, but got {actual:?}",
        );
        let actual = actual.unwrap();
        assert_eq!(
            expected, actual,
            "expected ApiSuccess {expected:?}, but got {actual:?}",
        );
    }
}
