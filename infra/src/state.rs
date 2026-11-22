use axum::extract::FromRef;
use leptos::config::LeptosOptions;
use sqlx::SqlitePool;

use crate::{auth::Keys, config::AppConfig};

/// 全局应用状态，注入 Axum 与 Leptos 上下文
#[derive(Clone, FromRef)]
pub struct AppState {
    /// 应用配置
    pub config: AppConfig,
    /// JWT 编解码密钥
    pub keys: Keys,
    /// Leptos 运行时选项
    pub leptos_options: LeptosOptions,
    /// 数据库连接池
    pub pool: SqlitePool,
}
