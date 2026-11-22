/// 领域错误：表达业务规则校验失败
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("用户名和密码不能为空")]
    MissingCredentials,
    #[error("用户名或密码错误")]
    WrongCredentials,
    #[error("未提供授权令牌")]
    MissingToken,
    #[error("无效的授权令牌")]
    InvalidToken,
    #[error("资源不存在：{0}")]
    NotFound(&'static str),
}
