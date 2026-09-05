//! Domain types for the Current Jobs screen. Plain data: `serde` only, no
//! Leptos / database dependencies, so both the browser and the server use them.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

/// One row in the Current Jobs table.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JobSummary {
    pub id: i64,
    pub array_job_id: Option<i64>,
    pub array_task_id: Option<String>,
    pub name: Option<String>,
    /// Display name (the part before `(` in Slurm's `UserId`).
    pub user: String,
    /// Full `UserId`, e.g. `alice(1234)`. Kept for exact filtering.
    pub user_raw: String,
    pub group: Option<String>,
    pub account: Option<String>,
    pub state: JobState,
    /// Raw Slurm state string, in case `state` collapsed it to `Other`.
    pub state_raw: String,
    /// Pending reason, e.g. `(Priority)`, `(Resources)`.
    pub reason: Option<String>,
    pub partition: Option<String>,
    pub qos: Option<String>,
    /// Human-readable node list, e.g. `virt-[3326-3327]`.
    pub nodelist: Option<String>,
    pub submit_time: Option<i64>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub runtime_secs: Option<f64>,
    pub time_limit_secs: Option<f64>,
    pub num_nodes: Option<i64>,
    pub num_cpus: i64,
    pub num_gpus: Option<i64>,
    /// Total requested memory, mebibytes.
    pub mem_mib: Option<i64>,
    /// Average GPU utilization while running, percent (0–100). `None` until the
    /// job has run.
    pub gpu_util: Option<f32>,
    pub cpu_util: Option<f32>,
    /// Peak memory utilization, percent of the allocation.
    pub mem_util: Option<f32>,
    pub exit_code: Option<String>,
}

/// Column the Current Jobs table can be sorted by. Purely a client-side
/// concern — sorting reorders the page already sitting in the browser, it
/// never re-queries the server (unlike the filters, which change the rows
/// that match and so do go back to the DB).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortKey {
    User,
    Id,
    Name,
    State,
    Runtime,
    TimeLimit,
    Partition,
    Gpus,
    Cpus,
    Mem,
    GpuUtil,
    CpuUtil,
    MemUtil,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDir {
    Asc,
    Desc,
}

impl SortDir {
    pub fn flipped(self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }

    /// Glyph for the active sort indicator in a column header.
    pub fn arrow(self) -> &'static str {
        match self {
            Self::Asc => "▲",
            Self::Desc => "▼",
        }
    }
}

impl JobSummary {
    /// Ordering for a click-to-sort table header. Missing values (e.g.
    /// utilization on a job that hasn't run yet) always sort last, in either
    /// direction, so they settle at the bottom instead of jumping to the top
    /// when the direction flips.
    pub fn cmp_by(&self, other: &Self, key: SortKey, dir: SortDir) -> Ordering {
        match key {
            SortKey::User => cmp_text(&self.user, &other.user, dir),
            SortKey::Id => cmp_ord(self.id, other.id, dir),
            SortKey::Name => cmp_text(
                self.name.as_deref().unwrap_or(""),
                other.name.as_deref().unwrap_or(""),
                dir,
            ),
            SortKey::State => cmp_text(self.state.label(), other.state.label(), dir),
            SortKey::Runtime => cmp_opt(self.runtime_secs, other.runtime_secs, dir),
            SortKey::TimeLimit => cmp_opt(self.time_limit_secs, other.time_limit_secs, dir),
            SortKey::Partition => cmp_text(
                self.partition.as_deref().unwrap_or(""),
                other.partition.as_deref().unwrap_or(""),
                dir,
            ),
            SortKey::Gpus => cmp_opt(self.num_gpus, other.num_gpus, dir),
            SortKey::Cpus => cmp_ord(self.num_cpus, other.num_cpus, dir),
            SortKey::Mem => cmp_opt(self.mem_mib, other.mem_mib, dir),
            SortKey::GpuUtil => cmp_opt(self.gpu_util, other.gpu_util, dir),
            SortKey::CpuUtil => cmp_opt(self.cpu_util, other.cpu_util, dir),
            SortKey::MemUtil => cmp_opt(self.mem_util, other.mem_util, dir),
        }
    }
}

fn cmp_ord<T: Ord>(a: T, b: T, dir: SortDir) -> Ordering {
    let o = a.cmp(&b);
    if dir == SortDir::Desc {
        o.reverse()
    } else {
        o
    }
}

fn cmp_text(a: &str, b: &str, dir: SortDir) -> Ordering {
    let o = a.to_lowercase().cmp(&b.to_lowercase());
    if dir == SortDir::Desc {
        o.reverse()
    } else {
        o
    }
}

/// `None` always sorts after every `Some`, in both directions.
fn cmp_opt<T: PartialOrd>(a: Option<T>, b: Option<T>, dir: SortDir) -> Ordering {
    match (a, b) {
        (Some(x), Some(y)) => {
            let o = x.partial_cmp(&y).unwrap_or(Ordering::Equal);
            if dir == SortDir::Desc {
                o.reverse()
            } else {
                o
            }
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// Slurm job state, bucketed into the values the UI cares about. Anything
/// unrecognised lands in [`JobState::Other`] with the raw string kept on
/// [`JobSummary::state_raw`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Running,
    Pending,
    Suspended,
    Completing,
    Configuring,
    Completed,
    Failed,
    Cancelled,
    Timeout,
    NodeFail,
    OutOfMemory,
    BootFail,
    Deadline,
    Preempted,
    Other,
}

/// Whether a state is still on the cluster or has ended. Drives whether the
/// finished-jobs time window applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobPhase {
    Active,
    Finished,
}

impl JobState {
    pub fn parse(s: &str) -> Self {
        // Slurm sometimes suffixes a reason, e.g. "CANCELLED by 1000".
        let head = s.split_whitespace().next().unwrap_or("").to_ascii_uppercase();
        match head.as_str() {
            "RUNNING" => Self::Running,
            "PENDING" => Self::Pending,
            "SUSPENDED" => Self::Suspended,
            "COMPLETING" => Self::Completing,
            "CONFIGURING" => Self::Configuring,
            "COMPLETED" => Self::Completed,
            "FAILED" => Self::Failed,
            "CANCELLED" | "CANCELLED+" => Self::Cancelled,
            "TIMEOUT" => Self::Timeout,
            "NODE_FAIL" => Self::NodeFail,
            "OUT_OF_MEMORY" | "OUT_OF_ME+" => Self::OutOfMemory,
            "BOOT_FAIL" => Self::BootFail,
            "DEADLINE" => Self::Deadline,
            "PREEMPTED" => Self::Preempted,
            _ => Self::Other,
        }
    }

    /// Canonical uppercase name (round-trips through [`JobState::parse`], except
    /// `Other`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "RUNNING",
            Self::Pending => "PENDING",
            Self::Suspended => "SUSPENDED",
            Self::Completing => "COMPLETING",
            Self::Configuring => "CONFIGURING",
            Self::Completed => "COMPLETED",
            Self::Failed => "FAILED",
            Self::Cancelled => "CANCELLED",
            Self::Timeout => "TIMEOUT",
            Self::NodeFail => "NODE_FAIL",
            Self::OutOfMemory => "OUT_OF_MEMORY",
            Self::BootFail => "BOOT_FAIL",
            Self::Deadline => "DEADLINE",
            Self::Preempted => "PREEMPTED",
            Self::Other => "OTHER",
        }
    }

    /// Lower-case, title-styled label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Pending => "pending",
            Self::Suspended => "suspended",
            Self::Completing => "completing",
            Self::Configuring => "configuring",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Timeout => "timeout",
            Self::NodeFail => "node fail",
            Self::OutOfMemory => "out of memory",
            Self::BootFail => "boot fail",
            Self::Deadline => "deadline",
            Self::Preempted => "preempted",
            Self::Other => "other",
        }
    }

    pub fn phase(&self) -> JobPhase {
        match self {
            Self::Running
            | Self::Pending
            | Self::Suspended
            | Self::Completing
            | Self::Configuring => JobPhase::Active,
            _ => JobPhase::Finished,
        }
    }

    pub fn is_active(&self) -> bool {
        self.phase() == JobPhase::Active
    }

    /// CSS modifier suffix for colour-coding, e.g. `state--running`.
    pub fn css_kind(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Pending | Self::Configuring => "pending",
            Self::Suspended | Self::Completing => "warn",
            Self::Completed => "ok",
            Self::Failed | Self::NodeFail | Self::BootFail | Self::OutOfMemory => "error",
            Self::Timeout | Self::Deadline | Self::Preempted => "warn",
            Self::Cancelled | Self::Other => "muted",
        }
    }

    /// The state chips offered in the filter bar, in display order.
    pub fn filter_choices() -> &'static [JobState] {
        &[
            Self::Running,
            Self::Pending,
            Self::Completed,
            Self::Failed,
            Self::Cancelled,
            Self::Timeout,
            Self::Suspended,
        ]
    }
}

/// Filter + paging options for the Current Jobs query. Round-trips to URL query
/// parameters so a filtered view is shareable and survives a reload.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct JobQuery {
    /// Uppercase Slurm state names. Empty ⇒ the server's configured default.
    #[serde(default)]
    pub states: Vec<String>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub job_id: Option<i64>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub partition: Option<String>,
    #[serde(default)]
    pub node: Option<String>,
    #[serde(default)]
    pub account: Option<String>,
    /// Override the finished-jobs window, hours. Clamped by the server.
    #[serde(default)]
    pub window_hours: Option<i64>,
    /// Override the row cap. Clamped by the server.
    #[serde(default)]
    pub limit: Option<i64>,
}

impl JobQuery {
    /// Build from decoded URL query pairs. Unknown keys are ignored; blank
    /// values are treated as absent.
    pub fn from_pairs<I, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut q = JobQuery::default();
        for (k, v) in pairs {
            let v = v.as_ref().trim();
            if v.is_empty() {
                continue;
            }
            match k.as_ref() {
                "state" | "states" => {
                    q.states = v
                        .split(',')
                        .map(|s| s.trim().to_ascii_uppercase())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                "user" => q.user = Some(v.to_string()),
                "job_id" | "id" => q.job_id = v.parse().ok(),
                "name" => q.name = Some(v.to_string()),
                "partition" => q.partition = Some(v.to_string()),
                "node" => q.node = Some(v.to_string()),
                "account" => q.account = Some(v.to_string()),
                "window" | "window_hours" => q.window_hours = v.parse().ok(),
                "limit" => q.limit = v.parse().ok(),
                _ => {}
            }
        }
        q
    }

    /// The inverse of [`JobQuery::from_pairs`]. Only non-empty fields are
    /// emitted, so a default query yields an empty vec.
    pub fn to_pairs(&self) -> Vec<(&'static str, String)> {
        let mut out: Vec<(&'static str, String)> = Vec::new();
        if !self.states.is_empty() {
            out.push(("state", self.states.join(",")));
        }
        if let Some(v) = &self.user {
            out.push(("user", v.clone()));
        }
        if let Some(v) = self.job_id {
            out.push(("id", v.to_string()));
        }
        if let Some(v) = &self.name {
            out.push(("name", v.clone()));
        }
        if let Some(v) = &self.partition {
            out.push(("partition", v.clone()));
        }
        if let Some(v) = &self.node {
            out.push(("node", v.clone()));
        }
        if let Some(v) = &self.account {
            out.push(("account", v.clone()));
        }
        if let Some(v) = self.window_hours {
            out.push(("window", v.to_string()));
        }
        if let Some(v) = self.limit {
            out.push(("limit", v.to_string()));
        }
        out
    }

    /// `"?a=b&c=d"`, or `""` when no filters are set. Values are percent-encoded
    /// for the few characters that matter in a query string.
    pub fn to_query_string(&self) -> String {
        let pairs = self.to_pairs();
        if pairs.is_empty() {
            return String::new();
        }
        let body = pairs
            .iter()
            .map(|(k, v)| format!("{k}={}", encode_component(v)))
            .collect::<Vec<_>>()
            .join("&");
        format!("?{body}")
    }

    pub fn has_state(&self, s: JobState) -> bool {
        self.states.iter().any(|x| x == s.as_str())
    }
}

/// Result of a Current Jobs query.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct JobPage {
    pub jobs: Vec<JobSummary>,
    /// `true` when the row cap was hit and more rows matched than were returned.
    pub truncated: bool,
    /// The row cap that was applied (after server-side clamping).
    pub limit: i64,
}

/// Minimal query-component encoder — enough for filter values (spaces, `&`,
/// `#`, `,`, non-ASCII). Avoids pulling in a URL crate on the client.
fn encode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Format a duration as Slurm does: `D-HH:MM:SS`, dropping the day part when 0.
pub fn format_duration(secs: f64) -> String {
    let total = secs.max(0.0) as i64;
    let (d, rem) = (total / 86_400, total % 86_400);
    let (h, rem) = (rem / 3_600, rem % 3_600);
    let (m, s) = (rem / 60, rem % 60);
    if d > 0 {
        format!("{d}-{h:02}:{m:02}:{s:02}")
    } else {
        format!("{h:02}:{m:02}:{s:02}")
    }
}

/// Mebibytes → a compact `GiB`/`MiB` string.
pub fn format_mem_mib(mib: i64) -> String {
    if mib >= 1024 {
        let gib = mib as f64 / 1024.0;
        if gib.fract() == 0.0 {
            format!("{gib:.0} GiB")
        } else {
            format!("{gib:.1} GiB")
        }
    } else {
        format!("{mib} MiB")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_round_trips() {
        for s in JobState::filter_choices() {
            assert_eq!(JobState::parse(s.as_str()), *s);
        }
    }

    #[test]
    fn state_parse_ignores_trailing_reason() {
        assert_eq!(JobState::parse("CANCELLED by 1000"), JobState::Cancelled);
    }

    #[test]
    fn query_pairs_round_trip() {
        let q = JobQuery {
            states: vec!["RUNNING".into(), "PENDING".into()],
            user: Some("alice".into()),
            job_id: Some(42),
            window_hours: Some(6),
            ..Default::default()
        };
        let back = JobQuery::from_pairs(q.to_pairs());
        assert_eq!(q, back);
    }

    #[test]
    fn blank_values_are_dropped() {
        let q = JobQuery::from_pairs([("user", ""), ("id", "  "), ("state", "running")]);
        assert_eq!(q.user, None);
        assert_eq!(q.job_id, None);
        assert_eq!(q.states, vec!["RUNNING".to_string()]);
    }

    #[test]
    fn default_query_has_empty_string() {
        assert_eq!(JobQuery::default().to_query_string(), "");
    }

    #[test]
    fn duration_formats_like_slurm() {
        assert_eq!(format_duration(3_600.0), "01:00:00");
        assert_eq!(format_duration(90_061.0), "1-01:01:01");
    }

    fn job(id: i64, user: &str, gpu_util: Option<f32>) -> JobSummary {
        JobSummary {
            id,
            array_job_id: None,
            array_task_id: None,
            name: None,
            user: user.into(),
            user_raw: user.into(),
            group: None,
            account: None,
            state: JobState::Running,
            state_raw: "RUNNING".into(),
            reason: None,
            partition: None,
            qos: None,
            nodelist: None,
            submit_time: None,
            start_time: None,
            end_time: None,
            runtime_secs: None,
            time_limit_secs: None,
            num_nodes: None,
            num_cpus: 1,
            num_gpus: None,
            mem_mib: None,
            gpu_util,
            cpu_util: None,
            mem_util: None,
            exit_code: None,
        }
    }

    #[test]
    fn sort_by_id_respects_direction() {
        let a = job(2, "b", None);
        let b = job(1, "a", None);
        assert_eq!(a.cmp_by(&b, SortKey::Id, SortDir::Asc), Ordering::Greater);
        assert_eq!(a.cmp_by(&b, SortKey::Id, SortDir::Desc), Ordering::Less);
    }

    #[test]
    fn sort_by_user_is_case_insensitive() {
        let a = job(1, "Bob", None);
        let b = job(2, "alice", None);
        assert_eq!(a.cmp_by(&b, SortKey::User, SortDir::Asc), Ordering::Greater);
    }

    #[test]
    fn missing_values_sort_last_in_both_directions() {
        let has_util = job(1, "a", Some(10.0));
        let no_util = job(2, "b", None);
        assert_eq!(
            has_util.cmp_by(&no_util, SortKey::GpuUtil, SortDir::Asc),
            Ordering::Less
        );
        assert_eq!(
            has_util.cmp_by(&no_util, SortKey::GpuUtil, SortDir::Desc),
            Ordering::Less
        );
    }
}
