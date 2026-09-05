use leptos::prelude::*;
use thaw::{Button, ButtonAppearance};

use crate::components::icons::IconSearch;

/// Slim header above the page content. The global search is decorative for now;
/// wire it to a command palette / route later.
#[component]
pub fn Topbar() -> impl IntoView {
    view! {
        <header class="topbar">
            <label class="topbar__search">
                <IconSearch />
                <input type="text" placeholder="Search…" disabled />
            </label>
            <Button appearance=ButtonAppearance::Primary>"Sign in"</Button>
        </header>
    }
}
