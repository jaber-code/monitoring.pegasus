use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_query_map};

use crate::api::get_current_jobs;
use crate::components::jobs::{JobsFilterBar, JobsTable};
use crate::components::layout::{AppShell, SidebarItem};
use crate::models::JobQuery;

/// Current Jobs screen: a filter bar over a live jobs table.
///
/// The filter state lives entirely in the URL query string, so a filtered view
/// is shareable and reload-safe. The bar reports edits; this component writes
/// them back to the URL, which re-drives the [`Resource`].
#[component]
pub fn CurrentJobsPage() -> impl IntoView {
    // Recognised filter keys, read straight off the URL query map.
    const KEYS: [&str; 9] = [
        "state", "user", "id", "name", "partition", "node", "account", "window", "limit",
    ];
    let params = use_query_map();
    let query = Memo::new(move |_| {
        params.with(|p| {
            let pairs: Vec<(String, String)> = KEYS
                .iter()
                .filter_map(|k| p.get(k).map(|v| (k.to_string(), v.to_string())))
                .collect();
            JobQuery::from_pairs(pairs)
        })
    });

    let navigate = use_navigate();
    let set = Callback::new(move |q: JobQuery| {
        navigate(
            &format!("/slurm/current-jobs{}", q.to_query_string()),
            Default::default(),
        );
    });

    let jobs = Resource::new(move || query.get(), get_current_jobs);
    let page = Signal::derive(move || jobs.get().and_then(|r| r.ok()).unwrap_or_default());

    view! {
        <AppShell active=SidebarItem::Dashboards>
            <section class="catalog">
                <header class="catalog__head">
                    <h1 class="catalog__title">"Current Jobs"</h1>
                    <p class="catalog__subtitle">
                        "Running, pending and recently finished Slurm jobs"
                    </p>
                </header>

                <JobsFilterBar query=query set=set />

                <Transition fallback=move || {
                    view! { <div class="catalog__note">"Loading jobs…"</div> }
                }>
                    {move || match jobs.get() {
                        None => ().into_any(),
                        Some(Err(e)) => {
                            view! {
                                <div class="catalog__note catalog__note--error">
                                    {format!("Could not load jobs: {e}")}
                                </div>
                            }
                                .into_any()
                        }
                        Some(Ok(_)) => view! { <JobsTable page=page /> }.into_any(),
                    }}
                </Transition>
            </section>
        </AppShell>
    }
}
