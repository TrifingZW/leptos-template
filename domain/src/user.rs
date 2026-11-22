/// 用户实体
#[derive(Debug, Clone, PartialEq, Eq)]
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
