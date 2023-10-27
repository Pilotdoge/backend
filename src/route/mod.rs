use serde::Serialize;
use crate::route::err::BackendError;

pub mod eligible;
pub mod err;
pub mod stat;
pub mod merkle;

#[derive(Debug, Serialize, Clone)]
pub struct BackendResponse<T: Clone + Serialize> {
    pub code: BackendError,
    pub error: Option<String>,
    pub data: Option<T>
}
