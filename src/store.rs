use crate::model::{
    Author, CreateAuthorError, CreateAuthorRequest, DeleteAuthorError, DeleteAuthorRequest,
    FindAllAuthorsError, FindAuthorError, FindAuthorRequest,
};
use async_trait::async_trait;

#[async_trait]
pub trait AuthorRepository: Send + Sync + 'static {
    async fn create_author(&self, req: &CreateAuthorRequest) -> Result<Author, CreateAuthorError>;

    async fn find_author(&self, req: &FindAuthorRequest) -> Result<Author, FindAuthorError>;

    async fn find_all_authors(&self) -> Result<Vec<Author>, FindAllAuthorsError>;

    async fn delete_author(&self, req: &DeleteAuthorRequest) -> Result<(), DeleteAuthorError>;
}
