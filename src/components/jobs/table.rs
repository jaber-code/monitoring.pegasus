use leptos::prelude::*;

use super::UtilBar;
use crate::models::{format_duration, format_mem_mib, JobPage, JobState, JobSummary, SortDir, SortKey};

/// The Current Jobs results table. Column set mirrors the Slurm "Current Jobs"
/// dashboard: identity, state, reason, timing, allocation, utilization.
///
/// Sorting is entirely client-side: `sort` picks a column out of the page
/// already in memory (see [`JobSummary::cmp_by`]) and re-orders it before
/// rendering. It never touches the filters or re-queries the server — that's
/// the job of [`crate::models::JobQuery`] up in [`crate::pages::CurrentJobsPage`].
#[component]
pub fn JobsTable(#[prop(into)] page: Signal<JobPage>) -> impl IntoView {
    // `None` = the server's order (state priority, then job id). Local UI
    // state: unlike the filters, a sort choice isn't worth putting in the URL.
    let sort = RwSignal::new(None::<(SortKey, SortDir)>);

    move || {
        let mut page = page.get();
        if page.jobs.is_empty() {
            return view! {
                <div class="catalog__note">"No jobs match these filters."</div>
            }
            .into_any();
        }

        if let Some((key, dir)) = sort.get() {
            page.jobs.sort_by(|a, b| a.cmp_by(b, key, dir));
        }

        let truncated = page.truncated.then_some(page.limit);
        let rows = page
            .jobs
            .into_iter()
            .map(|job| view! { <JobRow job=job /> })
            .collect_view();

        view! {
            <div class="jobs-table__wrap">
                <table class="jobs-table">
                    <thead>
                        <tr>
                            <SortableTh label="User" sort_key=SortKey::User sort=sort />
                            <SortableTh label="ID" sort_key=SortKey::Id sort=sort />
                            <SortableTh label="Name" sort_key=SortKey::Name sort=sort />
                            <SortableTh label="State" sort_key=SortKey::State sort=sort />
                            <th>"Reason"</th>
                            <SortableTh label="Runtime" sort_key=SortKey::Runtime sort=sort />
                            <SortableTh label="Time limit" sort_key=SortKey::TimeLimit sort=sort />
                            <SortableTh label="Partition" sort_key=SortKey::Partition sort=sort />
                            <SortableTh
                                label="GPU"
                                sort_key=SortKey::Gpus
                                sort=sort
                                numeric=true
                                title="Requested GPUs"
                            />
                            <SortableTh
                                label="CPU"
                                sort_key=SortKey::Cpus
                                sort=sort
                                numeric=true
                                title="Requested CPUs"
                            />
                            <SortableTh
                                label="Mem"
                                sort_key=SortKey::Mem
                                sort=sort
                                numeric=true
                                title="Requested memory"
                            />
                            <SortableTh label="GPU util" sort_key=SortKey::GpuUtil sort=sort />
                            <SortableTh label="CPU util" sort_key=SortKey::CpuUtil sort=sort />
                            <SortableTh label="Mem util" sort_key=SortKey::MemUtil sort=sort />
                        </tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
                {truncated
                    .map(|limit| {
                        view! {
                            <p class="jobs-table__note">
                                {format!(
                                    "Showing the first {limit} rows — narrow the filters to see more.",
                                )}
                            </p>
                        }
                    })}
            </div>
        }
        .into_any()
    }
}

/// A clickable column header: click to sort ascending, click again to flip to
/// descending, click a different header to switch columns (ascending).
#[component]
fn SortableTh(
    label: &'static str,
    sort_key: SortKey,
    sort: RwSignal<Option<(SortKey, SortDir)>>,
    #[prop(optional)] numeric: bool,
    #[prop(default = "")] title: &'static str,
) -> impl IntoView {
    let active_dir = move || sort.get().and_then(|(k, d)| (k == sort_key).then_some(d));

    let on_click = move |_| {
        sort.update(|current| {
            *current = match current {
                Some((k, d)) if *k == sort_key => Some((sort_key, d.flipped())),
                _ => Some((sort_key, SortDir::Asc)),
            };
        });
    };

    view! {
        <th
            class="jobs-table__sortable"
            class:num=numeric
            class:is-sorted=move || active_dir().is_some()
            title=title
            on:click=on_click
        >
            <span>{label}</span>
            <span class="jobs-table__sort-arrow">
                {move || active_dir().map(SortDir::arrow).unwrap_or("")}
            </span>
        </th>
    }
}

#[component]
fn JobRow(job: JobSummary) -> impl IntoView {
    let JobSummary {
        id,
        name,
        user,
        state,
        state_raw,
        reason,
        partition,
        runtime_secs,
        time_limit_secs,
        num_cpus,
        num_gpus,
        mem_mib,
        gpu_util,
        cpu_util,
        mem_util,
        ..
    } = job;

    let state_label = if state == JobState::Other {
        state_raw.to_lowercase()
    } else {
        state.label().to_string()
    };
    let state_class = format!("state state--{}", state.css_kind());

    // Reason: raw string kept for the tooltip, tidied text shown in the cell.
    // Mostly populated for PENDING jobs; blank elsewhere.
    let reason_raw = reason
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty() && !r.eq_ignore_ascii_case("none"));
    let reason_title = reason_raw.clone().unwrap_or_default();
    let reason_text = reason_raw.as_deref().map(tidy_reason).unwrap_or_default();

    // Runtime is a duration only: elapsed for anything that has run, else "--".
    let runtime_cell = match runtime_secs {
        Some(s) if s > 0.0 => view! { <span>{format_duration(s)}</span> }.into_any(),
        _ => view! { <span class="muted">"--"</span> }.into_any(),
    };

    let time_limit_cell = time_limit_secs
        .map(format_duration)
        .unwrap_or_else(|| "--".to_string());

    let name_text = name.unwrap_or_else(|| "—".to_string());
    let name_title = name_text.clone();
    let partition_text = partition.unwrap_or_default();
    let partition_title = partition_text.clone();
    let mem_text = mem_mib
        .map(format_mem_mib)
        .unwrap_or_else(|| "--".to_string());

    view! {
        <tr>
            <td>{user}</td>
            <td>
                <a class="jobs-table__id" href=format!("/jobs/{id}")>
                    {id.to_string()}
                </a>
            </td>
            <td class="jobs-table__name" title=name_title>{name_text}</td>
            <td>
                <span class=state_class>{state_label}</span>
            </td>
            <td class="jobs-table__reason" title=reason_title>{reason_text}</td>
            <td>{runtime_cell}</td>
            <td>{time_limit_cell}</td>
            <td class="jobs-table__partition" title=partition_title>
                {partition_text.clone()}
            </td>
            <td class="num">{num_gpus.map(|n| n.to_string()).unwrap_or_default()}</td>
            <td class="num">{num_cpus.to_string()}</td>
            <td class="num">{mem_text}</td>
            <td class="jobs-table__util">
                <UtilBar value=gpu_util kind="gpu" />
            </td>
            <td class="jobs-table__util">
                <UtilBar value=cpu_util kind="cpu" />
            </td>
            <td class="jobs-table__util">
                <UtilBar value=mem_util kind="mem" />
            </td>
        </tr>
    }
}

/// Slurm's pending `Reason` → short human text for the Reason column.
///
/// Slurm wraps multi-word reasons in parens and joins words with `_`. We strip
/// one paren layer, map the handful of common codes, and pass anything else
/// through with underscores turned to spaces. The raw string stays in the
/// cell's `title`, so nothing is lost.
fn tidy_reason(raw: &str) -> String {
    let s = raw.trim();
    let s = s
        .strip_prefix('(')
        .and_then(|inner| inner.strip_suffix(')'))
        .unwrap_or(s)
        .trim();
    match s {
        "Priority" => "priority".into(),
        "Resources" => "waiting for resources".into(),
        "Dependency" => "waiting on dependency".into(),
        "DependencyNeverSatisfied" => "dependency never satisfied".into(),
        "JobArrayTaskLimit" => "array task limit reached".into(),
        "BeginTime" => "start time not reached".into(),
        "ReqNodeNotAvail" => "required node unavailable".into(),
        "QOSMaxJobsPerUserLimit" => "QOS per-user job limit".into(),
        "" | "None" => String::new(),
        other => other.replace('_', " "),
    }
}
