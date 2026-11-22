use domain::DomainError;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};

use crate::{
    auth::claims::{Claims, ISSUER},
    error::AppError,
};

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
            .map_err(|_| DomainError::InvalidToken)?;

        Ok(data.claims)
    }
}

#[cfg(test)]
mod tests {
    use domain::User;
    use time::{Duration, OffsetDateTime};

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
            Err(AppError::Domain(DomainError::InvalidToken))
        ));
    }

    #[test]
    fn rejects_token_with_other_issuer() {
        let keys = Keys::new("test-secret").unwrap();
        let mut claims = Claims::for_user(&user(), Duration::hours(1), OffsetDateTime::now_utc());
        claims.iss = "other-issuer".to_string();
        let token = keys.encode(&claims).unwrap();

        assert!(matches!(keys.decode(&token), Err(AppError::Domain(DomainError::InvalidToken))));
    }

    #[test]
    fn rejects_expired_token() {
        let keys = Keys::new("test-secret").unwrap();
        let issued_at = OffsetDateTime::now_utc() - Duration::hours(2);
        let claims = Claims::for_user(&user(), Duration::hours(1), issued_at);
        let token = keys.encode(&claims).unwrap();

        assert!(matches!(keys.decode(&token), Err(AppError::Domain(DomainError::InvalidToken))));
    }

    #[test]
    fn rejects_empty_secret() {
        assert!(matches!(Keys::new(""), Err(AppError::Config(_))));
    }
}
