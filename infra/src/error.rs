use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use domain::DomainError;
use serde_json::json;

/// 应用统一错误类型
///
/// 作为 Axum 处理器错误时经 [`IntoResponse`] 输出对应状态码与 JSON；
/// 作为 Server Function 返回值时按 Leptos 约定收敛为 `ServerFnError`，
/// 客户端只取到错误消息，需要精确状态码的接口请使用 HTTP 处理器。
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error("配置错误：{0}")]
    Config(String),
    #[error("数据库错误：{0}")]
    Database(#[from] sqlx::Error),
    #[error("数据库迁移失败：{0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("令牌错误：{0}")]
    Token(#[from] jsonwebtoken::errors::Error),
    #[error("内部错误：{0}")]
    Internal(String),
}

impl AppError {
    /// 错误对应的 HTTP 状态码
    pub fn status(&self) -> StatusCode {
        match self {
            Self::Domain(error) => match error {
                DomainError::MissingCredentials => StatusCode::BAD_REQUEST,
                DomainError::NotFound(_) => StatusCode::NOT_FOUND,
                DomainError::WrongCredentials
                | DomainError::MissingToken
                | DomainError::InvalidToken => StatusCode::UNAUTHORIZED,
            },
            Self::Config(_)
            | Self::Database(_)
            | Self::Migration(_)
            | Self::Token(_)
            | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 1) 服务端错误记录完整细节，客户端只收到概要信息
        let status = self.status();
        if status.is_server_error() {
            tracing::error!(error = %self, "请求处理失败");
        }

        // 2) 输出统一 JSON 错误体
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
