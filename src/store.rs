use crate::model::{
    Author, CreateAuthorError, CreateAuthorRequest, FindAllAuthorsError, FindAuthorError,
    FindAuthorRequest,
};
use std::future::Future;

pub trait AuthorRepository: Clone + Send + Sync + 'static {
    fn create_author(
        &self,
        req: &CreateAuthorRequest,
    ) -> impl Future<Output = Result<Author, CreateAuthorError>> + Send;

    fn find_author(
        &self,
        req: &FindAuthorRequest,
    ) -> impl Future<Output = Result<Author, FindAuthorError>> + Send;

    fn find_all_authors(
        &self,
    ) -> impl Future<Output = Result<Vec<Author>, FindAllAuthorsError>> + Send;
}
