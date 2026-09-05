# Cluster Monitoring UI

A monitoring web app for a Slurm HPC cluster, built with **Leptos 0.8 (Rust, SSR) on
Axum** and **thaw** for components/theming. Job data is mirrored into **PostgreSQL**;
**Prometheus** metrics are wired up but not yet used by a screen. Screens so far: a
home / dashboard catalog and a Current Jobs table.

## Architecture

```
src/
├── app.rs, api.rs        route table; #[server] functions (client→server bridge)
├── config.rs, metrics.rs (ssr) config loading, Prometheus client
├── models/                shared domain types (unit-tested)
├── data/                  `ClusterData` trait + Mock / Live (Postgres) impls
├── db/                (ssr) Postgres pool + column-adaptive SQL for `jobs`
├── components/            layout chrome, catalog widgets, jobs widgets
└── pages/                 home, current_jobs, not_found
```

Data flow: `component → api::* (#[server]) → data::provider() → impl ClusterData
→ models`. `ClusterData` is the one trait every screen talks to. At startup the
server tries `config.yaml` + Postgres; if either is missing/unreachable it falls
back to an in-memory mock, so the UI always renders with no database.

Current Jobs' filter state lives in the URL query string (shareable, reload-safe);
the server always clamps rows/time-window per `config.yaml`, regardless of the
request.

## Running locally

```sh
cargo leptos watch     # no config.yaml needed — runs on mock data, http://127.0.0.1:3000
cargo test             # unit tests in models/
```

To use a real Postgres mirror, copy `config.example.yaml` to `config.yaml`.

## For reviewers

- Best entry points: `db/jobs.rs` (schema-adaptive SQL) and `models/*.rs` (the
  unit-tested logic) — more representative than the glue in `api.rs`/`pages/`.
- Worth discussing: the single `ClusterData` trait as the only UI↔backend seam,
  filter state in the URL instead of component state, server-side query clamping.
- Known gaps: no auth, no metrics screen yet, no e2e tests, no CI.

## Disclosure

Parts of this project were built with the help of Claude (Anthropic) as an AI
coding assistant.
