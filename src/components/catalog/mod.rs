//! The dashboard catalog: a search/tag toolbar over a list of collapsible
//! folders. Owns the client-side filter state; fetches its data via
//! [`crate::api::get_catalog`].

mod folder;
mod toolbar;

use leptos::prelude::*;

use crate::api::get_catalog;
use crate::models::Folder;
use folder::FolderList;
use toolbar::CatalogToolbar;

#[component]
pub fn DashboardCatalog() -> impl IntoView {
    // Server data. Serialised into the SSR response, reused on hydrate.
    let catalog = Resource::new(|| (), |_| get_catalog());

    // Client-side filter state.
    let search = RwSignal::new(String::new());
    let active_tags = RwSignal::new(Vec::<String>::new());

    // Derived views of the loaded catalog. Lazy: only evaluated when read from
    // inside the <Transition> below, so they never touch the resource early.
    let all_tags = Signal::derive(move || {
        catalog
            .get()
            .and_then(|r| r.ok())
            .map(|c| c.all_tags())
            .unwrap_or_default()
    });

    let filtered = Signal::derive(move || match catalog.get() {
        Some(Ok(cat)) => cat.filter(&search.get(), &active_tags.get()),
        _ => Vec::<Folder>::new(),
    });

    // While a filter is active, force every folder open (matches Grafana).
    let force_open =
        Signal::derive(move || !search.get().is_empty() || !active_tags.get().is_empty());

    view! {
        <section class="catalog">
            <header class="catalog__head">
                <h1 class="catalog__title">"Dashboards"</h1>
                <p class="catalog__subtitle">
                    "Browse dashboards for jobs, services and cluster resources"
                </p>
            </header>

            <Transition fallback=move || {
                view! { <div class="catalog__note">"Loading dashboards…"</div> }
            }>
                {move || match catalog.get() {
                    None => ().into_any(),
                    Some(Err(err)) => {
                        view! {
                            <div class="catalog__note catalog__note--error">
                                {format!("Could not load dashboards: {err}")}
                            </div>
                        }
                            .into_any()
                    }
                    Some(Ok(_)) => {
                        view! {
                            <CatalogToolbar
                                search=search
                                active_tags=active_tags
                                all_tags=all_tags
                            />
                            <FolderList folders=filtered force_open=force_open />
                        }
                            .into_any()
                    }
                }}
            </Transition>
        </section>
    }
}
