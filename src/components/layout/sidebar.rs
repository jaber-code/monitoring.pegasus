use leptos::prelude::*;

use crate::components::icons::{IconBell, IconDashboard, IconHome, IconStar};

/// Top-level navigation targets. Extend as new sections are added.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarItem {
    Home,
    Starred,
    #[default]
    Dashboards,
    Alerting,
}

#[component]
pub fn Sidebar(#[prop(optional)] active: SidebarItem) -> impl IntoView {
    view! {
        <aside class="sidebar">
            <div class="sidebar__brand">
                <span class="sidebar__logo">"◎"</span>
                <span>"Cluster Monitor"</span>
            </div>
            <nav class="sidebar__nav">
                <NavItem href="/" label="Home" active=active == SidebarItem::Home>
                    <IconHome />
                </NavItem>
                <NavItem href="/starred" label="Starred" active=active == SidebarItem::Starred>
                    <IconStar />
                </NavItem>
                <NavItem href="/" label="Dashboards" active=active == SidebarItem::Dashboards>
                    <IconDashboard />
                </NavItem>
                <NavItem href="/alerting" label="Alerting" active=active == SidebarItem::Alerting>
                    <IconBell />
                </NavItem>
            </nav>
        </aside>
    }
}

#[component]
fn NavItem(
    href: &'static str,
    label: &'static str,
    #[prop(optional)] active: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <a href=href class="sidebar__item" class:is-active=active>
            {children()}
            <span>{label}</span>
        </a>
    }
}
