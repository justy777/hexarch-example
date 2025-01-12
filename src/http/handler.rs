use crate::http::AppState;
use crate::model::{
    Author, AuthorName, AuthorNameError, CreateAuthorError, CreateAuthorRequest, EmailAddress,
    EmailAddressError,
};
use crate::store::AuthorRepository;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ApiSuccess<T: Serialize>(StatusCode, Json<ApiResponse<T>>);

impl<T: Serialize> ApiSuccess<T> {
    pub const fn new(status: StatusCode, data: T) -> Self {
        Self(status, Json(ApiResponse::new(status, data)))
    }
}

impl<T: Serialize> IntoResponse for ApiSuccess<T> {
    fn into_response(self) -> axum::response::Response {
        (self.0, self.1).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    status_code: u16,
    data: T,
}

impl<T: Serialize> ApiResponse<T> {
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
    Conflict(String),
    UnprocessableEntity(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::InternalServerError(msg) => {
                let status = StatusCode::INTERNAL_SERVER_ERROR;
                (status, Json(ApiResponse::new(status, msg))).into_response()
            }
            Self::Conflict(msg) => {
                let status = StatusCode::CONFLICT;
                (status, Json(ApiResponse::new(status, msg))).into_response()
            }
            Self::UnprocessableEntity(msg) => {
                let status = StatusCode::UNPROCESSABLE_ENTITY;
                (status, Json(ApiResponse::new(status, msg))).into_response()
            }
        }
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
                Self::Conflict(format!("author with name \"{name}\" already exists"))
            }
            CreateAuthorError::Unknown(cause) => {
                eprintln!("{cause}");
                eprintln!("{}", cause.backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
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

#[derive(Debug, Serialize)]
pub struct CreateAuthorHttpResponse {
    id: u64,
}

impl From<Author> for CreateAuthorHttpResponse {
    fn from(value: Author) -> Self {
        Self { id: value.id() }
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
