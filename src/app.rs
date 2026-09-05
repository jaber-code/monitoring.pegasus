//! Application shell: HTML document, global providers and the route table.

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path, StaticSegment,
};
use thaw::ssr::SSRMountStyleProvider;
use thaw::{ConfigProvider, Theme};

use crate::pages::{CurrentJobsPage, HomePage, NotFound};

/// The full HTML document rendered on the server for every request.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <SSRMountStyleProvider>
            <!DOCTYPE html>
            <html lang="en" data-theme="dark">
                <head>
                    <meta charset="utf-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1" />
                    <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
                    <AutoReload options=options.clone() />
                    <HydrationScripts options />
                    <MetaTags />
                </head>
                <body>
                    <App />
                </body>
            </html>
        </SSRMountStyleProvider>
    }
}

/// Root component: providers + router. Mounted on both server and client.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // thaw component theming. Kept in a signal so a theme switcher can be
    // dropped in later without touching this file.
    let theme = RwSignal::new(Theme::dark());

    view! {
        <Stylesheet id="leptos" href="/pkg/monitoring.css" />
        <Title text="Cluster Monitoring" />

        <ConfigProvider theme>
            <Router>
                <Routes fallback=|| view! { <NotFound /> }>
                    <Route path=StaticSegment("") view=HomePage />
                    <Route path=path!("slurm/current-jobs") view=CurrentJobsPage />
                // Future screens plug in here, e.g.
                //   <Route path=path!("jobs" / :id) view=JobDetailPage />
                </Routes>
            </Router>
        </ConfigProvider>
    }
}
