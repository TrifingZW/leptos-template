use leptos::prelude::*;
use ui::Button;

use crate::components::CurrentUserBadge;

/// 首页
#[component]
pub fn HomePage() -> impl IntoView {
    let count = RwSignal::new(0);

    view! {
        <h1>"Welcome to Leptos Template!"</h1>
        <Button on_click=Callback::new(move |_| *count.write() += 1)>"Click Me: " {count}</Button>
        <CurrentUserBadge/>
    }
}
