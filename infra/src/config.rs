use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

/// 配置文件路径环境变量
pub const CONFIG_PATH_ENV: &str = "APP_CONFIG";
/// 默认配置文件路径
pub const DEFAULT_CONFIG_PATH: &str = "config.toml";
/// 程序生成配置文件时写入的头部注释
const CONFIG_HEADER: &str = "# 应用配置：修改后重启生效\n\n";

/// 应用配置根节点
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// 认证与 JWT 会话配置
    pub auth: AuthConfig,
    /// 数据库连接配置
    pub database: DatabaseConfig,
    /// 服务器监听与 TLS 配置
    pub server: ServerConfig,
}

/// 认证与 JWT 会话配置
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    /// 签名密钥
    ///
    /// 首次生成的配置会写入随机值；已有配置留空时使用进程内临时密钥，
    /// 重启将使现有会话失效。
    pub jwt_secret: String,
    /// 令牌与 Cookie 的有效期（小时）
    pub jwt_ttl_hours: u64,
}

/// 数据库连接配置
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    /// 连接池最大连接数
    pub max_connections: u32,
    /// 数据库连接字符串
    pub url: String,
}

/// 服务器监听与 TLS 配置
///
/// 监听地址取自 `LeptosOptions::site_addr`（由 `LEPTOS_SITE_ADDR` 环境变量或
/// `Cargo.toml` 的 `site-addr` 提供），此处只配置 TLS 与 HTTP 重定向。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// 启用 TLS 时，HTTP 重定向监听端口
    pub http_redirect_port: u16,
    /// TLS 证书与私钥配置
    pub tls: TlsConfig,
}

/// TLS 证书配置
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TlsConfig {
    /// 是否启用 TLS
    pub enabled: bool,
    /// 证书文件路径
    pub cert: String,
    /// 私钥文件路径
    pub key: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: String::new(),
            jwt_ttl_hours: 24,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            max_connections: 5,
            url: "sqlite:leptos-template.db?mode=rwc".to_string(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            http_redirect_port: 80,
            tls: TlsConfig::default(),
        }
    }
}

impl AppConfig {
    /// 配置文件路径：优先取 [`CONFIG_PATH_ENV`]，否则使用当前目录下的默认文件名
    pub fn path() -> PathBuf {
        env::var_os(CONFIG_PATH_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH))
    }

    /// 从默认路径加载配置
    pub fn load() -> Result<Self, AppError> {
        Self::load_from(Self::path())
    }

    /// 从指定路径加载配置
    ///
    /// 文件缺失时写入带随机密钥的默认配置；密钥为空时使用进程内临时密钥，
    /// 不改写用户维护的配置文件。
    pub fn load_from(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let path = path.as_ref();

        // 1) 首次运行：生成带随机密钥的默认配置并落盘
        if !path.exists() {
            let config = Self {
                auth: AuthConfig {
                    jwt_secret: generate_secret(),
                    ..AuthConfig::default()
                },
                ..Self::default()
            };
            config.validate()?;
            config.save_to(path)?;
            tracing::info!(path = %path.display(), "已生成默认配置文件");

            return Ok(config);
        }

        // 2) 解析已有配置
        let mut config: Self = Self::parse(&Self::read(path)?, path)?;

        // 3) 缺少密钥时使用临时密钥，避免改写用户维护的配置文件
        if config.auth.jwt_secret.trim().is_empty() {
            config.auth.jwt_secret = generate_secret();
            tracing::warn!(
                path = %path.display(),
                "未配置 [auth].jwt_secret，已使用临时密钥，重启将使现有会话失效"
            );
        }

        // 4) 校验跨字段约束，尽早暴露配置错误
        config.validate()?;

        Ok(config)
    }

    /// 校验跨字段约束
    pub fn validate(&self) -> Result<(), AppError> {
        if self.auth.jwt_ttl_hours == 0 {
            return Err(AppError::Config("[auth].jwt_ttl_hours 必须大于 0".to_string()));
        }

        if self.database.url.trim().is_empty() {
            return Err(AppError::Config("[database].url 不能为空".to_string()));
        }

        if self.database.max_connections == 0 {
            return Err(AppError::Config("[database].max_connections 必须大于 0".to_string()));
        }

        if !self.server.tls.enabled {
            return Ok(());
        }

        if self.server.http_redirect_port == 0 {
            return Err(AppError::Config(
                "启用 TLS 时 [server].http_redirect_port 必须大于 0".to_string(),
            ));
        }

        for (field, path) in [("cert", &self.server.tls.cert), ("key", &self.server.tls.key)] {
            if path.trim().is_empty() {
                return Err(AppError::Config(format!("启用 TLS 时 [server.tls].{field} 不能为空")));
            }
            if !Path::new(path).is_file() {
                return Err(AppError::Config(format!(
                    "启用 TLS 时 [server.tls].{field} 文件不存在：{path}"
                )));
            }
        }

        Ok(())
    }

    /// 序列化为 TOML 文本
    pub fn to_toml(&self) -> Result<String, AppError> {
        toml::to_string(self).map_err(|e| AppError::Config(format!("序列化配置失败：{e}")))
    }

    /// 写回配置文件
    pub fn save_to(&self, path: impl AsRef<Path>) -> Result<(), AppError> {
        let path = path.as_ref();
        let content = format!("{CONFIG_HEADER}{}", self.to_toml()?);

        // 1) 确保父目录存在，避免自定义路径首次写入失败
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::Config(format!("创建配置目录 {} 失败：{e}", parent.display()))
            })?;
        }

        // 2) 写入完整配置
        fs::write(path, content)
            .map_err(|e| AppError::Config(format!("写入配置文件 {} 失败：{e}", path.display())))
    }

    fn read(path: &Path) -> Result<String, AppError> {
        fs::read_to_string(path)
            .map_err(|e| AppError::Config(format!("读取配置文件 {} 失败：{e}", path.display())))
    }

    fn parse(raw: &str, path: &Path) -> Result<Self, AppError> {
        toml::from_str(raw)
            .map_err(|e| AppError::Config(format!("解析配置文件 {} 失败：{e}", path.display())))
    }
}

/// 生成 64 位十六进制随机密钥
fn generate_secret() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_round_trips_through_toml() {
        let config = AppConfig::default();
        let parsed: AppConfig = toml::from_str(&config.to_toml().unwrap()).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn missing_sections_fall_back_to_defaults() {
        let parsed: AppConfig = toml::from_str("").unwrap();
        assert_eq!(parsed.database, DatabaseConfig::default());
        assert_eq!(parsed.server, ServerConfig::default());
        assert_eq!(parsed.auth, AuthConfig::default());
    }

    #[test]
    fn generated_secret_is_hex_and_unique() {
        let secret = generate_secret();
        assert_eq!(secret.len(), 64);
        assert!(secret.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(secret, generate_secret());
    }

    #[test]
    fn writes_and_reloads_default_config() {
        let path = env::temp_dir().join(format!("leptos-template-{}.toml", Uuid::new_v4()));
        let config = AppConfig::load_from(&path).unwrap();

        assert_eq!(config.database.url, "sqlite:leptos-template.db?mode=rwc");
        assert!(!config.auth.jwt_secret.is_empty());
        assert_eq!(AppConfig::load_from(&path).unwrap(), config);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_invalid_database_and_ttl_settings() {
        let mut config = AppConfig::default();
        config.auth.jwt_ttl_hours = 0;
        assert!(matches!(config.validate(), Err(AppError::Config(_))));

        let mut config = AppConfig::default();
        config.database.max_connections = 0;
        assert!(matches!(config.validate(), Err(AppError::Config(_))));

        let mut config = AppConfig::default();
        config.database.url = "  ".to_string();
        assert!(matches!(config.validate(), Err(AppError::Config(_))));
    }

    #[test]
    fn rejects_incomplete_tls_settings() {
        let mut config = AppConfig::default();
        config.server.tls.enabled = true;
        assert!(matches!(config.validate(), Err(AppError::Config(_))));

        config.server.tls.cert = "missing.crt".to_string();
        config.server.tls.key = "missing.key".to_string();
        assert!(matches!(config.validate(), Err(AppError::Config(_))));
    }
}
