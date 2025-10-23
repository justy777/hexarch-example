use regex::Regex;
use std::sync::LazyLock;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AuthorName(String);

impl AuthorName {
    pub fn new(raw: &str) -> Result<Self, AuthorNameEmptyError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            Err(AuthorNameEmptyError)
        } else {
            Ok(Self(trimmed.into()))
        }
    }

    pub fn new_unchecked(raw: &str) -> Self {
        Self(raw.into())
    }
}

impl std::fmt::Display for AuthorName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Error, Debug)]
#[error("Author name cannot be empty")]
pub struct AuthorNameEmptyError;

#[derive(Debug, Clone)]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn new(raw: &str) -> Result<Self, EmailAddressError> {
        let trimmed = raw.trim();
        if Self::is_valid(trimmed) {
            Ok(Self(trimmed.into()))
        } else {
            Err(EmailAddressError(trimmed.into()))
        }
    }

    pub fn new_unchecked(raw: &str) -> Self {
        Self(raw.into())
    }

    fn is_valid(s: &str) -> bool {
        static RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^[a-zA-Z0-9!#$%&'*+\-/=?^_`{|}~]+(.[a-zA-Z0-9!#$%&'*+\-/=?^_`{|}~]+)?@[a-zA-Z0-9]+(-[a-zA-Z0-9]+)?.[a-z]{2,3}$").unwrap()
        });
        RE.is_match(s)
    }
}

impl std::fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Error, Debug)]
#[error("{0} is not a valid email address")]
pub struct EmailAddressError(String);

#[derive(Debug)]
pub struct Author {
    id: i32,
    name: AuthorName,
    email: EmailAddress,
}

impl Author {
    pub const fn new(id: i32, name: AuthorName, email: EmailAddress) -> Self {
        Self { id, name, email }
    }

    pub const fn id(&self) -> i32 {
        self.id
    }

    pub const fn name(&self) -> &AuthorName {
        &self.name
    }

    pub const fn email(&self) -> &EmailAddress {
        &self.email
    }
}

#[derive(Debug)]
pub struct CreateAuthorRequest {
    name: AuthorName,
    email: EmailAddress,
}

impl CreateAuthorRequest {
    pub const fn new(name: AuthorName, email: EmailAddress) -> Self {
        Self { name, email }
    }

    pub const fn name(&self) -> &AuthorName {
        &self.name
    }

    pub const fn email(&self) -> &EmailAddress {
        &self.email
    }
}

#[derive(Error, Debug)]
pub enum CreateAuthorError {
    #[error("Author with name \"{name}\" already exists")]
    Duplicate { name: String },
    #[error(transparent)]
    Other(anyhow::Error),
}

#[derive(Debug)]
pub struct FindAuthorRequest {
    id: i32,
}

impl FindAuthorRequest {
    pub const fn new(id: i32) -> Self {
        Self { id }
    }

    pub const fn id(&self) -> i32 {
        self.id
    }
}

#[derive(Error, Debug)]
pub enum FindAuthorError {
    #[error("Author with id \"{id}\" does not exist")]
    NotFound { id: i32 },
    #[error(transparent)]
    Other(anyhow::Error),
}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct FindAllAuthorsError(#[from] pub anyhow::Error);

#[derive(Debug)]
pub struct UpdateAuthorRequest {
    id: i32,
    name: Option<AuthorName>,
    email: Option<EmailAddress>,
}

impl UpdateAuthorRequest {
    pub const fn new(id: i32) -> Self {
        Self {
            id,
            name: None,
            email: None,
        }
    }

    pub const fn id(&self) -> i32 {
        self.id
    }

    pub const fn name(&self) -> Option<&AuthorName> {
        self.name.as_ref()
    }

    pub fn set_name(&mut self, name: AuthorName) {
        self.name = Some(name);
    }

    pub const fn email(&self) -> Option<&EmailAddress> {
        self.email.as_ref()
    }

    pub fn set_email(&mut self, email: EmailAddress) {
        self.email = Some(email);
    }
}

#[derive(Error, Debug)]
pub enum UpdateAuthorError {
    #[error("Author with id \"{id}\" does not exist")]
    NotFound { id: i32 },
    #[error(transparent)]
    Other(anyhow::Error),
}

#[derive(Debug)]
pub struct DeleteAuthorRequest {
    id: i32,
}

impl DeleteAuthorRequest {
    pub const fn new(id: i32) -> Self {
        Self { id }
    }

    pub const fn id(&self) -> i32 {
        self.id
    }
}

#[derive(Error, Debug)]
pub enum DeleteAuthorError {
    #[error("Author with id \"{id}\" does not exist")]
    NotFound { id: i32 },
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
