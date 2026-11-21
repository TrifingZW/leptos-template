//! 接口层：客户端与服务端共用的 Server Functions。
//!
//! 本 crate 只承载 RPC 契约与传输无关的 DTO，具体实现位于 `backend`。

pub mod auth;
pub mod user;
