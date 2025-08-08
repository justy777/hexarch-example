use crate::http::AppState;
use crate::model::{
    Author, AuthorName, AuthorNameError, CreateAuthorError, CreateAuthorRequest, DeleteAuthorError,
    DeleteAuthorRequest, EmailAddress, EmailAddressError, FindAllAuthorsError, FindAuthorError,
    FindAuthorRequest,
};
use crate::store::AuthorRepository;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ApiSuccess<T: Serialize + PartialEq>(StatusCode, Json<ApiResponse<T>>);

impl<T: Serialize + PartialEq> ApiSuccess<T> {
    pub const fn new(status: StatusCode, data: T) -> Self {
        Self(status, Json(ApiResponse::new(status, data)))
    }
}

impl<T> PartialEq for ApiSuccess<T>
where
    T: Serialize + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1.0 == other.1.0
    }
}

impl<T: Serialize + PartialEq> IntoResponse for ApiSuccess<T> {
    fn into_response(self) -> axum::response::Response {
        (self.0, self.1).into_response()
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct ApiResponse<T> {
    status_code: u16,
    data: T,
}

impl<T: Serialize + PartialEq> ApiResponse<T> {
    const fn new(status: StatusCode, data: T) -> Self {
        Self {
            status_code: status.as_u16(),
            data,
        }
    }
}

#[derive(Debug)]
pub enum ApiError {
    InternalServerError(String),
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    UnprocessableEntity(String),
}

impl ApiError {
    const fn status(&self) -> StatusCode {
        match self {
            Self::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::InternalServerError(msg)
            | Self::BadRequest(msg)
            | Self::NotFound(msg)
            | Self::Conflict(msg)
            | Self::UnprocessableEntity(msg) => msg,
        };
        write!(f, "{msg}")
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status();
        let msg = format!("{self}");
        (status, Json(ApiResponse::new(status, msg))).into_response()
    }
}

impl From<ParseCreateAuthorHttpRequestError> for ApiError {
    fn from(err: ParseCreateAuthorHttpRequestError) -> Self {
        let message = err.to_string();
        Self::UnprocessableEntity(message)
    }
}

impl From<CreateAuthorError> for ApiError {
    fn from(err: CreateAuthorError) -> Self {
        match err {
            CreateAuthorError::Duplicate { name } => {
                Self::Conflict(format!(r#"author with name "{name}" already exists"#))
            }
            CreateAuthorError::Unknown(cause) => {
                tracing::error!("{cause:?}\n{}", cause.backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<FindAuthorError> for ApiError {
    fn from(err: FindAuthorError) -> Self {
        match err {
            FindAuthorError::NotFound { id } => {
                Self::NotFound(format!(r#"author with id "{id}" does not exist"#))
            }
            FindAuthorError::Unknown(cause) => {
                tracing::error!("{cause:?}\n{}", cause.backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<FindAllAuthorsError> for ApiError {
    fn from(err: FindAllAuthorsError) -> Self {
        match err {
            FindAllAuthorsError(cause) => {
                tracing::error!("{cause:?}\n{}", cause.backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<DeleteAuthorError> for ApiError {
    fn from(err: DeleteAuthorError) -> Self {
        match err {
            DeleteAuthorError::NotFound { id } => {
                Self::NotFound(format!(r#"author with id "{id}" does not exist"#))
            }
            DeleteAuthorError::Unknown(cause) => {
                tracing::error!("{cause:?}\n{}", cause.backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<ParseIdError> for ApiError {
    fn from(err: ParseIdError) -> Self {
        Self::BadRequest(format!(r#"Cannot parse id from "{}""#, err.id))
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateAuthorHttpRequest {
    name: String,
    email: String,
}

#[derive(Debug)]
pub enum ParseCreateAuthorHttpRequestError {
    Name(AuthorNameError),
    Email(EmailAddressError),
}

impl From<AuthorNameError> for ParseCreateAuthorHttpRequestError {
    fn from(err: AuthorNameError) -> Self {
        Self::Name(err)
    }
}

impl From<EmailAddressError> for ParseCreateAuthorHttpRequestError {
    fn from(err: EmailAddressError) -> Self {
        Self::Email(err)
    }
}

impl std::fmt::Display for ParseCreateAuthorHttpRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(err) => write!(f, "{err}"),
            Self::Email(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ParseCreateAuthorHttpRequestError {}

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

pub async fn create_author<AR: AuthorRepository>(
    State(state): State<AppState<AR>>,
    Json(body): Json<CreateAuthorHttpRequest>,
) -> Result<ApiSuccess<CreateAuthorHttpResponse>, ApiError> {
    let req = body.try_into()?;
    state
        .author_repo
        .create_author(&req)
        .await
        .map_err(ApiError::from)
        .map(|author| ApiSuccess::new(StatusCode::CREATED, author.into()))
}

pub async fn find_author<AR: AuthorRepository>(
    Path(id): Path<String>,
    State(state): State<AppState<AR>>,
) -> Result<ApiSuccess<FindAuthorHttpResponse>, ApiError> {
    let req = id.try_into()?;
    state
        .author_repo
        .find_author(&req)
        .await
        .map_err(ApiError::from)
        .map(|author| ApiSuccess::new(StatusCode::OK, author.into()))
}

pub async fn find_all_authors<AR: AuthorRepository>(
    State(state): State<AppState<AR>>,
) -> Result<ApiSuccess<FindAllAuthorsHttpResponse>, ApiError> {
    state
        .author_repo
        .find_all_authors()
        .await
        .map_err(ApiError::from)
        .map(|authors| ApiSuccess::new(StatusCode::OK, authors.into()))
}

pub async fn delete_author<AR: AuthorRepository>(
    Path(id): Path<String>,
    State(state): State<AppState<AR>>,
) -> Result<ApiSuccess<()>, ApiError> {
    let req = id.try_into()?;
    state
        .author_repo
        .delete_author(&req)
        .await
        .map_err(ApiError::from)
        .map(|()| ApiSuccess::new(StatusCode::NO_CONTENT, ()))
}

#[cfg(test)]
mod tests {
    use crate::http::AppState;
    use crate::http::handler::{
        ApiSuccess, CreateAuthorHttpRequest, CreateAuthorHttpResponse, FindAllAuthorsHttpResponse,
        FindAuthorHttpResponse, create_author, delete_author, find_all_authors, find_author,
    };
    use crate::model::{
        Author, AuthorName, CreateAuthorError, CreateAuthorRequest, DeleteAuthorError,
        DeleteAuthorRequest, EmailAddress, FindAllAuthorsError, FindAuthorError, FindAuthorRequest,
    };
    use crate::store::AuthorRepository;
    use anyhow::anyhow;
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
                create: Arc::new(Mutex::new(Err(CreateAuthorError::Unknown(anyhow!(
                    "substitute error"
                ))))),
                find: Arc::new(Mutex::new(Err(FindAuthorError::Unknown(anyhow!(
                    "substitute error"
                ))))),
                find_all: Arc::new(Mutex::new(Err(FindAllAuthorsError(anyhow!(
                    "substitute error"
                ))))),
                delete: Arc::new(Mutex::new(Err(DeleteAuthorError::Unknown(anyhow!(
                    "substitute error"
                ))))),
            }
        }
    }

    impl AuthorRepository for MockAuthorRepository {
        async fn create_author(
            &self,
            _: &CreateAuthorRequest,
        ) -> Result<Author, CreateAuthorError> {
            let mut guard = self.create.lock();
            let mut result = Err(CreateAuthorError::Unknown(anyhow!("substitute error")));
            mem::swap(guard.as_deref_mut().unwrap(), &mut result);
            result
        }

        async fn find_author(&self, _: &FindAuthorRequest) -> Result<Author, FindAuthorError> {
            let mut guard = self.find.lock();
            let mut result = Err(FindAuthorError::Unknown(anyhow!("substitute error")));
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
            let mut result = Err(DeleteAuthorError::Unknown(anyhow!("substitute error")));
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
        let expected = ApiSuccess::new(
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
        let expected = ApiSuccess::new(
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
        let expected = ApiSuccess::new(
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
        let expected = ApiSuccess::new(StatusCode::NO_CONTENT, ());
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
