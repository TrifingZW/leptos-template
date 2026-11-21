//! HTTP 中间件装配。

use axum::{Router, extract::DefaultBodyLimit};
use tower_http::limit::RequestBodyLimitLayer;

/// 请求体大小上限（250 MiB），用于支撑文件上传等场景
const MAX_BODY_BYTES: usize = 250 * 1024 * 1024;

/// 为路由挂载通用中间件
pub fn with_layers<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router.layer(DefaultBodyLimit::disable()).layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
}
