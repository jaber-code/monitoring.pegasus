//! The Current Jobs query.
//!
//! The SELECT list is built once at startup from the columns that actually
//! exist in `jobs` (see [`build_current_jobs_sql`]); a missing column becomes a
//! typed `NULL` so the statement always compiles. Filters are all
//! `$n IS NULL OR <predicate>`, so the shape — and the cached plan — is fixed
//! for the process. Guardrails from [`CurrentJobsConfig`] (row cap, finished
//! window, archived) are applied here, never trusted from the client.

use std::collections::HashSet;

use anyhow::Context;
use tokio_postgres::types::ToSql;
use tokio_postgres::Row;

use super::Db;
use crate::config::CurrentJobsConfig;
use crate::models::{JobPage, JobQuery, JobState, JobSummary};

impl Db {
    pub async fn current_jobs(
        &self,
        query: &JobQuery,
        defaults: &CurrentJobsConfig,
    ) -> anyhow::Result<JobPage> {
        // States: request wins, else the configured default.
        let states: Vec<String> = if query.states.is_empty() {
            defaults.default_states.clone()
        } else {
            query.states.clone()
        };

        // Clamp the guardrails.
        let limit = query
            .limit
            .unwrap_or(defaults.max_rows)
            .clamp(1, defaults.max_rows);
        let window_hours: i32 = query
            .window_hours
            .unwrap_or(defaults.finished_window_hours)
            .clamp(1, 24 * 366) as i32;
        let include_archived = defaults.include_archived;

        // Fetch one extra row to detect truncation without a COUNT(*).
        let fetch: i64 = limit + 1;

        let user = query.user.as_deref();
        let job_id = query.job_id;
        let name_pat = query.name.as_ref().map(|n| format!("%{n}%"));
        let partition = query.partition.as_deref();
        let node_pat = query.node.as_ref().map(|n| format!("%{n}%"));
        let account = query.account.as_deref();

        let params: [&(dyn ToSql + Sync); 10] = [
            &states,
            &user,
            &job_id,
            &name_pat,
            &partition,
            &node_pat,
            &account,
            &include_archived,
            &window_hours,
            &fetch,
        ];

        let client = self.client().await?;
        let mut rows = client
            .query(self.jobs_sql.as_str(), &params)
            .await
            .context("current_jobs SQL")?;

        let truncated = rows.len() as i64 > limit;
        rows.truncate(limit as usize);

        let jobs = rows.iter().map(row_to_summary).collect();

        Ok(JobPage {
            jobs,
            truncated,
            limit,
        })
    }
}

/// Build the Current Jobs SELECT for the columns that exist in `cols` (all
/// lower-case). Absent columns are substituted with typed `NULL`s / literals so
/// the statement is always valid; the ten bind parameters are always the same.
pub(super) fn build_current_jobs_sql(cols: &HashSet<String>) -> String {
    let has = |c: &str| cols.contains(c);
    // A quoted reference when the column exists, otherwise a typed placeholder.
    let col = |c: &str, absent: &str| -> String {
        if has(c) {
            format!("\"{c}\"")
        } else {
            absent.to_string()
        }
    };
    let epoch = |c: &str| -> String {
        if has(c) {
            format!("EXTRACT(EPOCH FROM \"{c}\")::float8")
        } else {
            "NULL::float8".to_string()
        }
    };

    let name_expr = col("jobname", "''");
    let partition_expr = col("partition", "NULL::text");
    let account_expr = col("account", "NULL::text");
    let nodelist_expr = if has("nodelist") {
        "\"nodelist\"::text".to_string()
    } else {
        "NULL::text".to_string()
    };
    let archived_expr = col("archived", "false");
    let endtime_expr = col("endtime", "NULL::timestamptz");

    format!(
        r#"
SELECT
  "jobid"                              AS id,
  {arrayjobid}                         AS array_job_id,
  {arraytaskid}                        AS array_task_id,
  {name}                               AS name,
  "userid"                             AS user_raw,
  split_part("userid", '(', 1)         AS user_name,
  {groupid}                            AS "group",
  {account}                            AS account,
  "jobstate"                           AS state_raw,
  {reason}                             AS reason,
  {partition}                          AS partition,
  {qos}                                AS qos,
  {nodelist}                           AS nodelist,
  {numnodes}                           AS num_nodes,
  {numcpus}                            AS num_cpus,
  {numgpus}                            AS num_gpus,
  {mbmem}                              AS mem_mib,
  {exitcode}                           AS exit_code,
  {submit_ts}                          AS submit_ts,
  {start_ts}                           AS start_ts,
  {end_ts}                             AS end_ts,
  {runtime_s}                          AS runtime_s,
  {timelimit_s}                        AS timelimit_s,
  {gpu_util}                           AS gpu_util,
  {cpu_util}                           AS cpu_util,
  {mem_util}                           AS mem_util
FROM jobs
WHERE split_part("jobstate", ' ', 1) = ANY($1)
  AND ($8 OR {archived_expr} = false)
  AND ($2::text IS NULL OR split_part("userid", '(', 1) = $2)
  AND ($3::int8 IS NULL OR "jobid" = $3)
  AND ($4::text IS NULL OR {name_expr} ILIKE $4)
  AND ($5::text IS NULL OR {partition_expr} = $5)
  AND ($6::text IS NULL OR {nodelist_expr} ILIKE $6)
  AND ($7::text IS NULL OR {account_expr} = $7)
  AND (
        "jobstate" IN ('RUNNING', 'PENDING', 'SUSPENDED', 'COMPLETING', 'CONFIGURING')
     OR {endtime_expr} IS NULL
     OR {endtime_expr} >= now() - make_interval(hours => $9::int)
  )
ORDER BY
  CASE
    WHEN "jobstate" = 'RUNNING'     THEN 0
    WHEN "jobstate" = 'COMPLETING'  THEN 1
    WHEN "jobstate" = 'CONFIGURING' THEN 2
    WHEN "jobstate" = 'PENDING'     THEN 3
    WHEN "jobstate" = 'SUSPENDED'   THEN 4
    ELSE 5
  END,
  "jobid" DESC
LIMIT $10
"#,
        arrayjobid = col("arrayjobid", "NULL::int8"),
        arraytaskid = col("arraytaskid", "NULL::text"),
        name = name_expr,
        groupid = col("groupid", "NULL::text"),
        account = account_expr,
        reason = col("reason", "NULL::text"),
        partition = partition_expr,
        qos = col("qos", "NULL::text"),
        nodelist = nodelist_expr,
        numnodes = col("numnodes", "NULL::text"),
        numcpus = col("numcpus", "0::int8"),
        numgpus = col("numgpus", "NULL::int8"),
        mbmem = col("mbmem", "NULL::int8"),
        exitcode = col("exitcode", "NULL::text"),
        submit_ts = epoch("submittime"),
        start_ts = epoch("starttime"),
        end_ts = epoch("endtime"),
        runtime_s = epoch("runtime"),
        timelimit_s = epoch("timelimit"),
        gpu_util = col("avggpuutilization", "NULL::float4"),
        cpu_util = col("avgcpuutilization", "NULL::float4"),
        mem_util = col("maxmemutilization", "NULL::float4"),
    )
}

fn row_to_summary(row: &Row) -> JobSummary {
    let state_raw: String = row.get("state_raw");
    let user_name: String = row.get("user_name");
    let user_raw: String = row.get("user_raw");

    JobSummary {
        id: row.get::<_, i64>("id"),
        array_job_id: row.get("array_job_id"),
        array_task_id: non_empty(row.get("array_task_id")),
        name: non_empty(row.get("name")),
        user: if user_name.is_empty() {
            user_raw.clone()
        } else {
            user_name
        },
        user_raw,
        group: non_empty(row.get("group")),
        account: non_empty(row.get("account")),
        state: JobState::parse(&state_raw),
        state_raw: state_raw.clone(),
        reason: non_empty(row.get("reason")).filter(|r| !r.eq_ignore_ascii_case("none")),
        partition: non_empty(row.get("partition")),
        qos: non_empty(row.get("qos")),
        nodelist: row
            .get::<_, Option<String>>("nodelist")
            .and_then(|s| clean_nodelist(&s)),
        submit_time: to_epoch(row.get("submit_ts")),
        start_time: to_epoch(row.get("start_ts")),
        end_time: to_epoch(row.get("end_ts")),
        runtime_secs: row.get("runtime_s"),
        time_limit_secs: row.get("timelimit_s"),
        num_nodes: row
            .get::<_, Option<String>>("num_nodes")
            .and_then(|s| s.trim().parse().ok()),
        num_cpus: row.get::<_, i64>("num_cpus"),
        num_gpus: row.get("num_gpus"),
        mem_mib: row.get("mem_mib"),
        gpu_util: row.get("gpu_util"),
        cpu_util: row.get("cpu_util"),
        mem_util: row.get("mem_util"),
        exit_code: non_empty(row.get("exit_code")),
    }
}

fn non_empty(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn to_epoch(secs: Option<f64>) -> Option<i64> {
    secs.map(|s| s as i64)
}

/// `nodelist` may be plain text (`virt-[1-2]`) or jsonb rendered as `["a","b"]`.
/// Strip any JSON punctuation down to `a,b`.
fn clean_nodelist(raw: &str) -> Option<String> {
    let s = raw.trim();
    if s.is_empty() || s == "null" || s == "[]" || s == "\"\"" {
        return None;
    }
    if !s.starts_with('[') && !s.starts_with('"') {
        return Some(s.to_string());
    }
    let inner = s.trim_start_matches('[').trim_end_matches(']');
    let joined = inner
        .split(',')
        .map(|p| p.trim().trim_matches('"').trim())
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(",");
    (!joined.is_empty()).then_some(joined)
}
