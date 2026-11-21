use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use crate::{db::User, error::AppError};

/// 令牌签发者标识
const ISSUER: &str = "leptos-template";

/// HMAC 编解码密钥对
#[derive(Clone)]
pub struct Keys {
    decoding: DecodingKey,
    encoding: EncodingKey,
}

impl Keys {
    /// 基于 HMAC 密钥构建编解码密钥
    pub fn new(secret: &str) -> Result<Self, AppError> {
        if secret.is_empty() {
            return Err(AppError::Config("签名密钥不能为空".to_string()));
        }

        Ok(Self {
            decoding: DecodingKey::from_secret(secret.as_bytes())?,
            encoding: EncodingKey::from_secret(secret.as_bytes())?,
        })
    }

    /// 签发令牌
    pub fn encode(&self, claims: &Claims) -> Result<String, AppError> {
        Ok(encode(&Header::new(Algorithm::HS256), claims, &self.encoding)?)
    }

    /// 校验并解析令牌
    pub fn decode(&self, token: &str) -> Result<Claims, AppError> {
        // 1) 固定算法并校验签发者，拒绝其他来源的同密钥令牌
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[ISSUER]);

        // 2) 解析失败统一收敛为无效令牌，避免向客户端泄露校验细节
        let data = decode::<Claims>(token, &self.decoding, &validation)
            .map_err(|_| AppError::InvalidToken)?;

        Ok(data.claims)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn user() -> User {
        User {
            id: 7,
            password: "secret".to_string(),
            role: "admin".to_string(),
            username: "admin".to_string(),
        }
    }

    #[test]
    fn encode_decode_round_trip() {
        let keys = Keys::new("test-secret").unwrap();
        let claims = Claims::for_user(&user(), Duration::hours(1), OffsetDateTime::now_utc());
        let token = keys.encode(&claims).unwrap();

        assert_eq!(keys.decode(&token).unwrap(), claims);
    }

    #[test]
    fn rejects_token_signed_with_other_secret() {
        let claims = Claims::for_user(&user(), Duration::hours(1), OffsetDateTime::now_utc());
        let token = Keys::new("test-secret").unwrap().encode(&claims).unwrap();

        assert!(matches!(
            Keys::new("other-secret").unwrap().decode(&token),
            Err(AppError::InvalidToken)
        ));
    }

    #[test]
    fn rejects_token_with_other_issuer() {
        let keys = Keys::new("test-secret").unwrap();
        let mut claims = Claims::for_user(&user(), Duration::hours(1), OffsetDateTime::now_utc());
        claims.iss = "other-issuer".to_string();
        let token = keys.encode(&claims).unwrap();

        assert!(matches!(keys.decode(&token), Err(AppError::InvalidToken)));
    }

    #[test]
    fn rejects_expired_token() {
        let keys = Keys::new("test-secret").unwrap();
        let issued_at = OffsetDateTime::now_utc() - Duration::hours(2);
        let claims = Claims::for_user(&user(), Duration::hours(1), issued_at);
        let token = keys.encode(&claims).unwrap();

        assert!(matches!(keys.decode(&token), Err(AppError::InvalidToken)));
    }

    #[test]
    fn rejects_empty_secret() {
        assert!(matches!(Keys::new(""), Err(AppError::Config(_))));
    }
}
