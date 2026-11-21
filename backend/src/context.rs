use axum::{extract::FromRequestParts, http::request::Parts};
use leptos::context::use_context;
use leptos_axum::ResponseOptions;

use crate::{auth::Claims, error::AppError, state::AppState};

/// 取当前请求的应用状态
pub fn state() -> Result<AppState, AppError> {
    use_context::<AppState>().ok_or_else(|| AppError::Internal("AppState 未注入上下文".to_string()))
}

/// 取响应写入句柄
pub fn response_options() -> Result<ResponseOptions, AppError> {
    use_context::<ResponseOptions>()
        .ok_or_else(|| AppError::Internal("ResponseOptions 未注入上下文".to_string()))
}

/// 取当前请求的 Axum 分段
pub fn request_parts() -> Result<Parts, AppError> {
    use_context::<Parts>().ok_or_else(|| AppError::Internal("请求 Parts 未注入上下文".to_string()))
}

/// 校验请求中的令牌并返回声明
pub async fn claims() -> Result<Claims, AppError> {
    let mut parts = request_parts()?;
    Claims::from_request_parts(&mut parts, &()).await
}
