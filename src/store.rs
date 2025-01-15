use crate::model::{
    Author, CreateAuthorError, CreateAuthorRequest, GetAuthorError, GetAuthorRequest,
};
use std::future::Future;

pub trait AuthorRepository: Clone + Send + Sync + 'static {
    fn create_author(
        &self,
        req: &CreateAuthorRequest,
    ) -> impl Future<Output = Result<Author, CreateAuthorError>> + Send;

    fn get_author(
        &self,
        req: &GetAuthorRequest,
    ) -> impl Future<Output = Result<Author, GetAuthorError>> + Send;
}
