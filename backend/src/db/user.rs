use sqlx::SqlitePool;

use crate::error::AppError;

/// `user` 表行模型
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    /// 主键
    pub id: i64,
    /// 密码（模板为明文，生产环境应存哈希）
    pub password: String,
    /// 角色
    pub role: String,
    /// 登录用户名
    pub username: String,
}

impl User {
    /// 按用户名查询用户
    pub async fn find_by_username(
        pool: &SqlitePool,
        username: &str,
    ) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as::<_, Self>(
            "SELECT id, password, role, username FROM user WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }
}
