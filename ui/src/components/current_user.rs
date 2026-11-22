use api::user::{CurrentUser, get_current_user};
use leptos::prelude::*;

/// 当前登录用户徽标，未登录时降级为提示文案
#[component]
pub fn CurrentUserBadge() -> impl IntoView {
    let current_user = Resource::new(|| (), |_| get_current_user());

    view! {
        <Suspense fallback=|| view! { <p>"加载用户信息…"</p> }>
            {move || {
                Suspend::new(async move {
                    match current_user.await {
                        Ok(CurrentUser { username, role }) => {
                            view! { <p>{format!("{username}（{role}）")}</p> }.into_any()
                        }
                        Err(_) => view! { <p>"未登录"</p> }.into_any(),
                    }
                })
            }}
        </Suspense>
    }
}
