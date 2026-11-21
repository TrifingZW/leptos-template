use leptos::prelude::*;

/// 路由未命中页面
#[component]
pub fn NotFound() -> impl IntoView {
    view! { <h1>"页面不存在"</h1> }
}
