use domain::User;
use sqlx::{FromRow, SqlitePool};

use crate::error::AppError;

/// `user` 表行映射
#[derive(FromRow)]
struct UserRow {
    id: i64,
    password: String,
    role: String,
    username: String,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            password: row.password,
            role: row.role,
            username: row.username,
        }
    }
}

/// 按用户名查询用户
pub async fn find_by_username(pool: &SqlitePool, username: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, password, role, username FROM user WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?
    .map(User::from);

    Ok(user)
}
