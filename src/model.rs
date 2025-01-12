pub struct Author {
    id: u64,
    name: String,
    email: String,
}

impl Author {
    pub const fn new(id: u64, name: String, email: String) -> Self {
        Self { id, name, email }
    }
}

pub struct CreateAuthorRequest {
    name: String,
    email: String,
}

impl CreateAuthorRequest {
    pub const fn new(name: String, email: String) -> Self {
        Self { name, email }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }
}

pub enum CreateAuthorError {
    Duplicate { name: String },
    Unknown(anyhow::Error),
}
