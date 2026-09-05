// Leptos view types nest deeply; SSR layout resolution needs headroom.
#![recursion_limit = "512"]
//! Cluster monitoring web UI.
//!
//! Crate layout
//! ------------
//! * [`app`]        – the shell, routing table and top-level `App` component.
//! * [`pages`]      – one module per screen (currently just the home screen).
//! * [`components`] – reusable UI: app chrome (sidebar / topbar) and the
//!                    dashboard catalog widgets.
//! * [`models`]     – plain, serialisable domain types shared by client & server.
//! * [`data`]       – server-side data access: the [`data::ClusterData`] trait
//!                    and its implementations (today a mock, tomorrow Prometheus
//!                    / slurmrestd).
//! * [`api`]        – `#[server]` functions; the only bridge the client uses to
//!                    reach [`data`].
//!
//! Adding a screen: add a `models` type + a `ClusterData` method + an `api`
//! function + a `pages` module + a `<Route>` in [`app`].

pub mod api;
pub mod app;
pub mod components;
pub mod data;
pub mod models;
pub mod pages;

#[cfg(feature = "ssr")]
pub mod config;
#[cfg(feature = "ssr")]
pub mod db;
#[cfg(feature = "ssr")]
pub mod metrics;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(crate::app::App);
}
