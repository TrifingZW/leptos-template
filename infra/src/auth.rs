//! 认证：令牌编解码、请求提取与会话管理。

mod claims;
mod extract;
mod keys;
mod session;

pub use claims::Claims;
pub use keys::Keys;
pub use session::{login, logout};

/// 会话 Cookie 名称
pub(crate) const SESSION_COOKIE: &str = "auth_token";
