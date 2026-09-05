//! Inline-SVG icons. Each renders a 16x16 glyph that inherits `currentColor`,
//! so colour/size is controlled from CSS via the `.icon` class.

use leptos::prelude::*;

#[component]
pub fn IconChevron() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6">
            <path d="M6 3.5 L10.5 8 L6 12.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
    }
}

#[component]
pub fn IconFolder() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3">
            <path
                d="M1.75 4.5c0-.7.55-1.25 1.25-1.25h2.8c.4 0 .78.19 1.02.5l.7.94H13c.7 0 1.25.56 1.25 1.25v6c0 .7-.55 1.25-1.25 1.25H3c-.7 0-1.25-.55-1.25-1.25v-7.4Z"
                stroke-linejoin="round"
            />
        </svg>
    }
}

#[component]
pub fn IconDashboard() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 16 16" fill="currentColor">
            <rect x="1.5" y="1.5" width="5.5" height="5.5" rx="1" />
            <rect x="9" y="1.5" width="5.5" height="5.5" rx="1" />
            <rect x="1.5" y="9" width="5.5" height="5.5" rx="1" />
            <rect x="9" y="9" width="5.5" height="5.5" rx="1" />
        </svg>
    }
}

#[component]
pub fn IconStar() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 1.6l1.9 3.98 4.35.6-3.15 3.02.77 4.4L8 11.94 4.13 13.6l.77-4.4L1.75 6.18l4.35-.6z" />
        </svg>
    }
}

#[component]
pub fn IconSearch() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="7" cy="7" r="4.5" />
            <path d="M10.5 10.5 L14 14" stroke-linecap="round" />
        </svg>
    }
}

#[component]
pub fn IconHome() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4">
            <path d="M2.5 7.5 8 2.5l5.5 5v6H2.5z" stroke-linejoin="round" />
        </svg>
    }
}

#[component]
pub fn IconBell() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4">
            <path d="M4 7a4 4 0 0 1 8 0c0 3 1 4 1 4H3s1-1 1-4z" stroke-linejoin="round" />
            <path d="M6.5 13a1.5 1.5 0 0 0 3 0" stroke-linecap="round" />
        </svg>
    }
}
