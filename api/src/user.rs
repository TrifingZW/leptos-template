use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// 当前登录用户的视图模型
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentUser {
    /// 用户角色
    pub role: String,
    /// 用户名
    pub username: String,
}

/// 读取当前登录用户
#[server]
pub async fn get_current_user() -> Result<CurrentUser, ServerFnError> {
    let claims = infra::context::claims().await?;

    Ok(CurrentUser {
        role: claims.role,
        username: claims.username,
    })
}
