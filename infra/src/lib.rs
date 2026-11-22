//! 基础设施层：配置、状态、认证、持久化与日志。
//!
//! 本 crate 只在 SSR 构建中参与编译，客户端（wasm）不得依赖它。

pub mod auth;
pub mod config;
pub mod context;
pub mod db;
pub mod error;
pub mod state;
pub mod telemetry;

pub use config::AppConfig;
pub use domain::User;
pub use error::AppError;
pub use state::AppState;
