use domain::User;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

/// 令牌签发者标识
pub(crate) const ISSUER: &str = "leptos-template";

/// JWT 声明
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// 过期时间（Unix 秒）
    pub exp: i64,
    /// 签发时间（Unix 秒）
    pub iat: i64,
    /// 签发者
    pub iss: String,
    /// 用户角色
    pub role: String,
    /// 用户 ID
    pub sub: String,
    /// 用户名
    pub username: String,
}

impl Claims {
    /// 依据用户与有效期构建声明
    pub fn for_user(user: &User, ttl: Duration, issued_at: OffsetDateTime) -> Self {
        let iat = issued_at.unix_timestamp();

        Self {
            exp: iat.saturating_add(ttl.whole_seconds()),
            iat,
            iss: ISSUER.to_string(),
            role: user.role.clone(),
            sub: user.id.to_string(),
            username: user.username.clone(),
        }
    }
}
