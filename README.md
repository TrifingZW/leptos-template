# leptos-template

Leptos + Axum SSR 模板，按职责划分为七个 crate：

| Crate | 职责 | 编译目标 |
| --- | --- | --- |
| `domain` | 纯业务模型与领域错误，不依赖基础设施 | 通用 |
| `infra` | 配置、数据库、JWT、会话与日志等基础设施适配 | 服务端 |
| `api` | 客户端与服务端共用的 Server Functions 与 DTO | 通用 |
| `ui` | 客户端与服务端共用的组件与页面 | 通用 |
| `app` | HTML 骨架、元信息与路由装配 | 通用 |
| `server` | 服务端二进制、中间件与 HTTP/HTTPS 监听 | 服务端 |
| `frontend` | wasm 水合入口 | 浏览器 |

依赖方向：`server` / `frontend` → `app` → `ui` → `api` → `infra` → `domain`（`server` 同时直接依赖 `infra`）。
