use leptos::prelude::*;

use crate::components::icons::{IconChevron, IconDashboard, IconFolder, IconStar};
use crate::models::{Dashboard, Folder};

/// Renders the filtered folder list, or an empty-state note.
///
/// The list is rebuilt whenever `folders` changes (i.e. on every keystroke in
/// the search box). That is cheap at catalog scale; the trade-off is that a
/// folder's manual expand/collapse state resets when the filter changes. While
/// a filter is active `force_open` is set, so this is invisible in practice.
#[component]
pub fn FolderList(
    #[prop(into)] folders: Signal<Vec<Folder>>,
    #[prop(into)] force_open: Signal<bool>,
) -> impl IntoView {
    move || {
        let folders = folders.get();
        if folders.is_empty() {
            view! { <div class="catalog__note">"No dashboards match your filters."</div> }.into_any()
        } else {
            view! {
                <div class="folder-list">
                    {folders
                        .into_iter()
                        .map(|folder| view! { <FolderSection folder=folder force_open=force_open /> })
                        .collect_view()}
                </div>
            }
                .into_any()
        }
    }
}

#[component]
fn FolderSection(folder: Folder, #[prop(into)] force_open: Signal<bool>) -> impl IntoView {
    let Folder { name, dashboards, .. } = folder;
    let count = dashboards.len();
    let open = RwSignal::new(true);
    let is_open = Memo::new(move |_| force_open.get() || open.get());

    // Rows are static; expand/collapse just toggles a class.
    let rows = dashboards
        .into_iter()
        .map(|d| view! { <DashboardRow dashboard=d /> })
        .collect_view();

    view! {
        <div class="folder">
            <button
                class="folder__header"
                class:is-open=move || is_open.get()
                on:click=move |_| open.update(|o| *o = !*o)
            >
                <span class="folder__chevron">
                    <IconChevron />
                </span>
                <IconFolder />
                <span class="folder__name">{name}</span>
                <span class="folder__count">{count} " dashboards"</span>
            </button>
            <div class="folder__body" class:is-hidden=move || !is_open.get()>
                {rows}
            </div>
        </div>
    }
}

#[component]
fn DashboardRow(dashboard: Dashboard) -> impl IntoView {
    let Dashboard { title, slug, tags, starred, route, .. } = dashboard;
    let href = route.unwrap_or_else(|| format!("/d/{slug}"));

    view! {
        <a class="dashboard-row" href=href>
            <span class="dashboard-row__icon">
                <IconDashboard />
            </span>
            <span class="dashboard-row__title">{title}</span>
            <span class="dashboard-row__tags">
                {tags
                    .into_iter()
                    .map(|t| view! { <span class="chip">{t}</span> })
                    .collect_view()}
            </span>
            <span class="dashboard-row__spacer"></span>
            <span class="dashboard-row__star" class:is-on=starred>
                <IconStar />
            </span>
        </a>
    }
}
