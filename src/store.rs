use crate::model::{Author, CreateAuthorError, CreateAuthorRequest};
use std::future::Future;

pub trait AuthorRepository: Clone + Send + Sync + 'static {
    fn create_author(
        &self,
        req: CreateAuthorRequest,
    ) -> impl Future<Output = Result<Author, CreateAuthorError>> + Send;
}
