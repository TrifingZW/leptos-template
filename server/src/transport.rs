//! HTTP/HTTPS 监听。

use std::net::SocketAddr;

use axum::{
    Router,
    extract::Request,
    http::{header::HOST, uri::Authority},
    response::Redirect,
};
use backend::config::{ServerConfig, TlsConfig};
use tracing::info;

/// 依据配置启动监听
pub async fn serve(app: Router, addr: SocketAddr, config: &ServerConfig) -> anyhow::Result<()> {
    // 1) 未启用 TLS 时直接提供 HTTP 服务
    if !config.tls.enabled {
        return http(app, addr).await;
    }

    // 2) 同时启动 HTTP 重定向与 HTTPS 服务，任一失败即退出
    tokio::try_join!(redirect(addr, config.http_redirect_port), https(app, addr, &config.tls))?;

    Ok(())
}

/// 启动 HTTP 服务
async fn http(app: Router, addr: SocketAddr) -> anyhow::Result<()> {
    info!("HTTP 服务已启动：http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}

/// 启动 HTTP → HTTPS 重定向服务
async fn redirect(https_addr: SocketAddr, http_port: u16) -> anyhow::Result<()> {
    let app = Router::new().fallback(move |request: Request| {
        let https_port = https_addr.port();
        async move { redirect_to_https(request, https_port) }
    });
    let addr = SocketAddr::new(https_addr.ip(), http_port);
    info!("HTTP 重定向服务已启动：http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}

/// 将请求重定向到 HTTPS
///
/// Host 头只保留主机名，端口统一替换为 HTTPS 实际监听端口，避免代理或
/// 非标准端口场景下重定向到错误地址。
fn redirect_to_https(request: Request, https_port: u16) -> Redirect {
    // 1) 提取 Host，解析失败时回退到 localhost
    let host = request
        .headers()
        .get(HOST)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<Authority>().ok())
        .map_or_else(|| "localhost".to_string(), |authority| authority.host().to_string());

    // 2) 仅非 443 端口显式追加端口
    let authority = if https_port == 443 {
        host
    } else {
        format!("{host}:{https_port}")
    };
    let path = request.uri().path_and_query().map_or("/", |pq| pq.as_str());

    Redirect::permanent(&format!("https://{authority}{path}"))
}

/// 启动 HTTPS 服务
async fn https(app: Router, addr: SocketAddr, tls: &TlsConfig) -> anyhow::Result<()> {
    let config = axum_server::tls_openssl::OpenSSLConfig::from_pem_file(&tls.cert, &tls.key)?;
    info!("HTTPS 服务已启动：https://{addr}");
    axum_server::bind_openssl(addr, config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}
