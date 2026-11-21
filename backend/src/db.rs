//! 持久化：连接池、迁移与数据表模型。

pub mod user;

pub use user::User;

use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

use crate::{config::DatabaseConfig, error::AppError};

/// 连接数据库并执行迁移
pub async fn connect(config: &DatabaseConfig) -> Result<SqlitePool, AppError> {
    // 1) 建立连接池；连接数至少为 1，避免 0 配置导致连接池不可用
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections.max(1))
        .connect(&config.url)
        .await?;

    // 2) 执行 migrations 目录下的全部迁移
    sqlx::migrate!("../migrations").run(&pool).await?;

    Ok(pool)
}
