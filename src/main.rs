#![recursion_limit = "512"]
//! Server entry point (SSR build only).

/// Stack size for the main thread and every tokio worker.
///
/// Leptos' view types nest very deeply, so both route discovery
/// (`generate_route_list`, which renders the whole app once) and per-request
/// SSR rendering need far more than the platform-default 1 MB stack — otherwise
/// the server thread overflows its stack and the socket closes with no response.
#[cfg(feature = "ssr")]
const STACK_SIZE: usize = 32 * 1024 * 1024;

#[cfg(feature = "ssr")]
fn main() {
    std::thread::Builder::new()
        .name("server-main".into())
        .stack_size(STACK_SIZE)
        .spawn(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_stack_size(STACK_SIZE)
                .build()
                .expect("build tokio runtime")
                .block_on(run());
        })
        .expect("spawn server-main thread")
        .join()
        .expect("server-main thread panicked");
}

#[cfg(feature = "ssr")]
async fn run() {
    use std::sync::Arc;

    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use monitoring::app::{shell, App};
    use monitoring::config::Config;
    use monitoring::data::{init_provider, LiveClusterData};
    use monitoring::metrics::Metrics;
    use tower_http::trace::{self, TraceLayer};
    use tracing::Level;
    use tracing_subscriber::{fmt, EnvFilter};

    // --- logging -----------------------------------------------------------
    // Request/response lines + our own `tracing` events go to the terminal.
    // Tune with RUST_LOG, e.g. `RUST_LOG=monitoring=debug,tower_http=debug`.
    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("info,monitoring=debug,tower_http=info")
        }))
        .compact()
        .init();

    // --- data source ---------------------------------------------------------
    // config.yaml present + DB reachable  ⇒  real Postgres backend.
    // Otherwise fall back to the in-memory mock so the UI still runs.
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            log!("data: config error ({e}); using mock data");
            None
        }
    };

    let mut pg_connected = false;
    match &config {
        Some(cfg) => match LiveClusterData::connect(cfg).await {
            Ok(live) => {
                log!(
                    "data: postgres {}@{}/{}",
                    cfg.database.user,
                    cfg.database.host,
                    cfg.database.dbname
                );
                init_provider(Arc::new(live));
                pg_connected = true;
            }
            Err(e) => log!("data: postgres unavailable ({e}); using mock data"),
        },
        None => log!("data: no config.yaml; using mock data"),
    }

    // Prometheus lives behind the same VPN as Postgres and its own private CA;
    // `LiveClusterData::connect` already probes it on the happy path. When the
    // DB is down we still probe here, so a VPN / certificate problem is visible
    // in the log instead of hidden behind the DB failure.
    if !pg_connected {
        if let Some(prom) = config.as_ref().and_then(|c| c.prometheus.as_ref()) {
            match Metrics::connect(prom) {
                Ok(m) => match m.health().await {
                    Ok(()) => log!("metrics: prometheus reachable at {}", prom.endpoint),
                    Err(e) => log!("metrics: prometheus unreachable ({e})"),
                },
                Err(e) => log!("metrics: prometheus client build failed ({e})"),
            }
        }
    }

    // --- http --------------------------------------------------------------
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        // One INFO line per response (method, path, status, latency); 5xx at
        // ERROR. Raise verbosity with `RUST_LOG=tower_http=debug`.
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
                .on_failure(trace::DefaultOnFailure::new().level(Level::ERROR)),
        )
        .with_state(leptos_options);

    log!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // The client is a WASM library (see `lib.rs::hydrate`); nothing to run here.
}
