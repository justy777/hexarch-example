use crate::http::AppState;
use crate::model::{
    Author, AuthorName, AuthorNameError, CreateAuthorError, CreateAuthorRequest, EmailAddress,
    EmailAddressError, GetAuthorError, GetAuthorRequest,
};
use crate::store::AuthorRepository;
use axum::extract::{Json, Path, State};
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

impl From<GetAuthorError> for ApiError {
    fn from(err: GetAuthorError) -> Self {
        match err {
            GetAuthorError::NotFound { id } => {
                Self::NotFound(format!(r#"author with id "{id}" does not exist"#))
            }
            GetAuthorError::Unknown(cause) => {
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

#[derive(Debug, Serialize)]
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

impl TryFrom<String> for GetAuthorRequest {
    type Error = ParseIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let id = value
            .parse::<u64>()
            .map_err(|_| ParseIdError { id: value })?;
        Ok(Self::new(id))
    }
}

#[derive(Debug, Serialize)]
pub struct GetAuthorHttpResponse {
    id: u64,
    name: String,
    email: String,
}

impl From<Author> for GetAuthorHttpResponse {
    fn from(value: Author) -> Self {
        Self {
            id: value.id(),
            name: value.name().to_string(),
            email: value.email().to_string(),
        }
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

pub async fn get_author<AR: AuthorRepository>(
    Path(id): Path<String>,
    State(state): State<AppState<AR>>,
) -> Result<ApiSuccess<GetAuthorHttpResponse>, ApiError> {
    let req = id.try_into()?;
    state
        .author_repo
        .get_author(&req)
        .await
        .map_err(ApiError::from)
        .map(|author| ApiSuccess::new(StatusCode::OK, author.into()))
}
