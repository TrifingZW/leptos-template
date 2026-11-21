use axum::http::{HeaderValue, header::SET_COOKIE};
use axum_extra::extract::cookie::{Cookie, SameSite};
use leptos_axum::ResponseOptions;
use time::{Duration, OffsetDateTime};

use crate::{
    auth::{Claims, SESSION_COOKIE},
    context,
    db::User,
    error::AppError,
};

/// 校验凭据、签发令牌并写入会话 Cookie
pub async fn login(username: &str, password: &str) -> Result<(), AppError> {
    // 1) 拒绝空凭据
    let username = username.trim();
    if username.is_empty() || password.is_empty() {
        return Err(AppError::MissingCredentials);
    }

    // 2) 校验用户与密码
    let state = context::state()?;
    let user =
        User::find_by_username(&state.pool, username).await?.ok_or(AppError::WrongCredentials)?;
    if !verify_password(password, &user.password) {
        return Err(AppError::WrongCredentials);
    }

    // 3) 依据配置的有效期签发令牌
    let ttl = token_ttl(state.config.auth.jwt_ttl_hours);
    let claims = Claims::for_user(&user, ttl, OffsetDateTime::now_utc());
    let token = state.keys.encode(&claims)?;

    // 4) 写入会话 Cookie
    let cookie = Cookie::build((SESSION_COOKIE, token))
        .http_only(true)
        .max_age(ttl)
        .path("/")
        .same_site(SameSite::Lax)
        .secure(is_https())
        .build();

    set_cookie(&context::response_options()?, &cookie)
}

/// 清除会话 Cookie
pub fn logout() -> Result<(), AppError> {
    let cookie = Cookie::build((SESSION_COOKIE, ""))
        .http_only(true)
        .max_age(Duration::ZERO)
        .path("/")
        .same_site(SameSite::Lax)
        .secure(is_https())
        .build();

    set_cookie(&context::response_options()?, &cookie)
}

/// 校验明文密码
///
/// 模板为简化演示保留明文比对；生产环境应改用 Argon2 或 bcrypt 校验密码哈希。
fn verify_password(input: &str, stored: &str) -> bool {
    input == stored
}

/// 将配置的小时数转换为令牌有效期
///
/// `Duration::hours` 在超大取值下会溢出 panic，改用秒并做饱和运算。
fn token_ttl(hours: u64) -> Duration {
    Duration::seconds(i64::try_from(hours).unwrap_or(i64::MAX).saturating_mul(3_600))
}

/// 请求是否经由 HTTPS 抵达
fn is_https() -> bool {
    // 1) 本机终止 TLS 时由配置判定
    if context::state().is_ok_and(|state| state.config.server.tls.enabled) {
        return true;
    }

    // 2) 反向代理转发时读取标准头
    context::request_parts()
        .ok()
        .and_then(|parts| {
            parts
                .headers
                .get("x-forwarded-proto")
                .and_then(|value| value.to_str().ok())
                .map(str::to_ascii_lowercase)
        })
        .is_some_and(|proto| proto == "https")
}

/// 将 Cookie 追加到响应头
fn set_cookie(response: &ResponseOptions, cookie: &Cookie<'_>) -> Result<(), AppError> {
    let value = HeaderValue::from_str(&cookie.to_string())
        .map_err(|e| AppError::Internal(format!("Cookie 头无效：{e}")))?;
    response.append_header(SET_COOKIE, value);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_ttl_saturates_huge_values() {
        assert_eq!(token_ttl(u64::MAX).whole_seconds(), i64::MAX);
    }
}
