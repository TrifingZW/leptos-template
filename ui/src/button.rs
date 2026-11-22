use leptos::prelude::*;

/// 基础按钮
///
/// 仅负责呈现与点击回调，不感知任何业务状态。
#[component]
pub fn Button(
    /// 点击回调
    #[prop(optional)]
    on_click: Option<Callback<()>>,
    /// 按钮内容
    children: Children,
) -> impl IntoView {
    view! {
        <button type="button" on:click=move |_| {
            if let Some(on_click) = on_click {
                on_click.run(());
            }
        }>
            {children()}
        </button>
    }
}
