use leptos::prelude::*;

use super::{Sidebar, SidebarItem, Topbar};

/// Page frame: fixed sidebar + topbar, with the screen's content in `<main>`.
///
/// ```ignore
/// view! { <AppShell active=SidebarItem::Dashboards> <MyScreen/> </AppShell> }
/// ```
#[component]
pub fn AppShell(
    /// Which sidebar entry to highlight.
    #[prop(optional)]
    active: SidebarItem,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="layout">
            <Sidebar active=active />
            <div class="layout__content">
                <Topbar />
                <main class="layout__main">{children()}</main>
            </div>
        </div>
    }
}
