use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug)]
pub struct AuthorName(String);

impl AuthorName {
    pub fn new(raw: &str) -> Result<Self, AuthorNameError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            Err(AuthorNameError::Empty)
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

#[derive(Debug)]
pub enum AuthorNameError {
    Invalid(String),
    Empty,
}

impl std::fmt::Display for AuthorNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(name) => write!(f, "Author name {name} is invalid"),
            Self::Empty => write!(f, "Author name cannot be empty"),
        }
    }
}

impl std::error::Error for AuthorNameError {}

#[derive(Debug)]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn new(raw: &str) -> Result<Self, EmailAddressError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            Err(EmailAddressError::Empty)
        } else if !Self::is_valid(trimmed) {
            Err(EmailAddressError::Invalid(trimmed.into()))
        } else {
            Ok(Self(trimmed.into()))
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

#[derive(Debug)]
pub enum EmailAddressError {
    Invalid(String),
    Empty,
}

impl std::fmt::Display for EmailAddressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(name) => write!(f, "Email address name {name} is invalid"),
            Self::Empty => write!(f, "Email address cannot be empty"),
        }
    }
}

impl std::error::Error for EmailAddressError {}

#[derive(Debug)]
pub struct Author {
    id: u64,
    name: AuthorName,
    email: EmailAddress,
}

impl Author {
    pub const fn new(id: u64, name: AuthorName, email: EmailAddress) -> Self {
        Self { id, name, email }
    }

    pub const fn id(&self) -> u64 {
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

#[derive(Debug)]
pub enum CreateAuthorError {
    Duplicate { name: String },
    Unknown(anyhow::Error),
}

impl std::fmt::Display for CreateAuthorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Duplicate { name } => write!(f, "Author with name \"{name}\" already exists"),
            Self::Unknown(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for CreateAuthorError {}
