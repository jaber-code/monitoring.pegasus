use leptos::prelude::*;

use crate::components::layout::AppShell;

#[component]
pub fn NotFound() -> impl IntoView {
    // On SSR, set the HTTP status to 404.
    #[cfg(feature = "ssr")]
    {
        let resp = expect_context::<leptos_axum::ResponseOptions>();
        resp.set_status(axum::http::StatusCode::NOT_FOUND);
    }

    view! {
        <AppShell>
            <section class="catalog">
                <header class="catalog__head">
                    <h1 class="catalog__title">"Page not found"</h1>
                    <p class="catalog__subtitle">"That page does not exist (yet)."</p>
                </header>
                <a class="link" href="/">"Back to dashboards"</a>
            </section>
        </AppShell>
    }
}
