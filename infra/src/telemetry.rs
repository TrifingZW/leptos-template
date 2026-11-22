//! 日志与追踪初始化。

use std::{io, path::PathBuf};

use time::macros::format_description;
use tracing_appender::{non_blocking, non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{
    EnvFilter, Registry, fmt, fmt::time::LocalTime, layer::SubscriberExt, util::SubscriberInitExt,
};

/// 日志文件目录
const LOG_DIR: &str = "logs";
/// 日志文件名前缀
const LOG_PREFIX: &str = "leptos-template.log";

/// 初始化全局日志记录器
///
/// 输出到控制台与按日滚动的文件；返回的守卫必须保留至进程退出，以确保日志刷盘。
pub fn init() -> WorkerGuard {
    // 1) 本地时区时间格式
    let timer =
        LocalTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"));

    // 2) 过滤器：默认 info，可由 RUST_LOG 覆盖
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // 3) 文件层：按日滚动、无 ANSI、记录线程 ID 与目标模块
    let (writer, guard) = non_blocking(rolling::daily(PathBuf::from(LOG_DIR), LOG_PREFIX));
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_timer(timer.clone())
        .with_writer(writer);

    // 4) 控制台层：启用 ANSI，精简目标与线程信息
    let console_layer = fmt::layer()
        .with_ansi(true)
        .with_target(false)
        .with_thread_ids(false)
        .with_timer(timer)
        .with_writer(io::stdout);

    // 5) 注册全局订阅者
    Registry::default().with(env_filter).with(file_layer).with(console_layer).init();

    tracing::info!("日志系统已就绪");

    guard
}
