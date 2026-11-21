use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    TypedHeader,
    extract::CookieJar,
    headers::{Authorization, authorization::Bearer},
};

use crate::{
    auth::{Claims, SESSION_COOKIE},
    context,
    error::AppError,
};

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = extract_token(parts).await.ok_or(AppError::MissingToken)?;
        let keys = context::state()?.keys;

        keys.decode(&token)
    }
}

/// 从 Cookie 或 `Authorization: Bearer` 头提取令牌
async fn extract_token(parts: &mut Parts) -> Option<String> {
    // 1) 浏览器会话优先读取 HttpOnly Cookie
    if let Ok(jar) = CookieJar::from_request_parts(parts, &()).await
        && let Some(cookie) = jar.get(SESSION_COOKIE)
    {
        return Some(cookie.value().to_string());
    }

    // 2) API 调用回退到 Authorization 头
    if let Ok(TypedHeader(Authorization(bearer))) =
        TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, &()).await
    {
        return Some(bearer.token().to_string());
    }

    None
}
