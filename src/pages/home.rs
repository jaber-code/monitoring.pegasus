use leptos::prelude::*;

use crate::components::catalog::DashboardCatalog;
use crate::components::layout::{AppShell, SidebarItem};

/// Home screen: the dashboard catalog inside the app shell.
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <AppShell active=SidebarItem::Dashboards>
            <DashboardCatalog />
        </AppShell>
    }
}
