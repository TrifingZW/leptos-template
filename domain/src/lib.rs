//! 领域层：纯业务模型与错误，不依赖任何基础设施。

pub mod error;
pub mod user;

pub use error::DomainError;
pub use user::User;
