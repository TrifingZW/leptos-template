//! 服务端入口：装配配置、数据库、路由与 HTTP/HTTPS 监听。

mod middleware;
mod telemetry;
mod transport;

use std::fs;

use anyhow::anyhow;
use axum::Router;
use backend::{AppConfig, AppState, auth::Keys, db};
use leptos::{config::LeptosOptions, prelude::provide_context};
use leptos_axum::{LeptosRoutes, generate_route_list};

use crate::{middleware::with_layers, transport::serve};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1) 初始化日志，守卫需存活至进程退出
    let _guard = telemetry::init();

    // 2) 加载应用配置与 Leptos 选项
    let config = AppConfig::load()?;
    let leptos_options = resolve_leptos_options()?;

    // 3) 确保站点根目录存在，缺失会导致静态资源与 404 处理失败
    fs::create_dir_all(leptos_options.site_root.as_ref())?;

    // 4) 连接数据库并执行迁移
    let pool = db::connect(&config.database).await?;

    // 5) 构建 JWT 密钥与全局状态
    let keys = Keys::new(&config.auth.jwt_secret)?;
    let state = AppState {
        config,
        keys,
        leptos_options: leptos_options.clone(),
        pool,
    };

    // 6) 装配 Leptos 路由与服务端上下文
    let routes = generate_route_list(app::App);
    let router = Router::new()
        .leptos_routes_with_context(
            &state,
            routes,
            {
                let state = state.clone();
                move || provide_context(state.clone())
            },
            {
                let leptos_options = leptos_options.clone();
                move || app::shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(app::shell))
        .with_state(state.clone());

    // 7) 挂载中间件并启动监听
    serve(with_layers(router), state.leptos_options.site_addr, &state.config.server).await
}

/// 解析 Leptos 运行时选项
///
/// 优先读取工作区 `Cargo.toml` 元数据（`LEPTOS_*` 环境变量仍然覆盖），
/// 部署环境没有 `Cargo.toml` 时回退为纯环境变量。
fn resolve_leptos_options() -> anyhow::Result<LeptosOptions> {
    leptos::config::get_configuration(Some("Cargo.toml"))
        .or_else(|_| leptos::config::get_configuration(None))
        .map(|configuration| configuration.leptos_options)
        .map_err(|e| anyhow!("Leptos 配置解析失败：{e}"))
}
