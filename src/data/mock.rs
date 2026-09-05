//! In-memory [`ClusterData`] used for local development and tests.
//!
//! The catalog mirrors the folder layout in the Grafana screenshots; the job
//! list is a small synthetic sample covering every state so the Current Jobs
//! screen is usable with no database.

use async_trait::async_trait;

use crate::data::{ClusterData, DataError};
use crate::models::{Catalog, Dashboard, Folder, JobPage, JobQuery, JobState, JobSummary};

pub struct MockClusterData {
    catalog: Catalog,
    jobs: Vec<JobSummary>,
}

impl MockClusterData {
    pub fn new() -> Self {
        Self {
            catalog: seed_catalog(),
            jobs: seed_jobs(),
        }
    }
}

impl Default for MockClusterData {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ClusterData for MockClusterData {
    async fn catalog(&self) -> Result<Catalog, DataError> {
        Ok(self.catalog.clone())
    }

    async fn current_jobs(&self, query: JobQuery) -> Result<JobPage, DataError> {
        let states = if query.states.is_empty() {
            vec![
                "RUNNING".to_string(),
                "PENDING".to_string(),
                "COMPLETED".to_string(),
                "FAILED".to_string(),
            ]
        } else {
            query.states.clone()
        };

        let limit = query.limit.unwrap_or(500).clamp(1, 500);

        let mut jobs: Vec<JobSummary> = self
            .jobs
            .iter()
            .filter(|j| states.iter().any(|s| s == j.state.as_str()))
            .filter(|j| query.job_id.map_or(true, |id| j.id == id))
            .filter(|j| {
                query
                    .user
                    .as_ref()
                    .map_or(true, |u| j.user.eq_ignore_ascii_case(u))
            })
            .filter(|j| {
                query.name.as_ref().map_or(true, |n| {
                    j.name
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&n.to_lowercase())
                })
            })
            .filter(|j| {
                query
                    .partition
                    .as_ref()
                    .map_or(true, |p| j.partition.as_deref() == Some(p.as_str()))
            })
            .filter(|j| {
                query.node.as_ref().map_or(true, |n| {
                    j.nodelist
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&n.to_lowercase())
                })
            })
            .cloned()
            .collect();

        let truncated = jobs.len() as i64 > limit;
        jobs.truncate(limit as usize);

        Ok(JobPage {
            jobs,
            truncated,
            limit,
        })
    }
}

fn dashboard(title: &str, tags: &[&str], starred: bool, route: Option<&str>) -> Dashboard {
    let slug = slugify(title);
    Dashboard {
        id: slug.clone(),
        title: title.to_string(),
        slug,
        tags: tags.iter().map(|t| t.to_string()).collect(),
        description: None,
        starred,
        route: route.map(|r| r.to_string()),
    }
}

fn folder(name: &str, dashboards: Vec<Dashboard>) -> Folder {
    Folder {
        id: slugify(name),
        name: name.to_string(),
        dashboards,
    }
}

fn slugify(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub(crate) fn seed_catalog() -> Catalog {
    Catalog {
        folders: vec![
            folder(
                "Cluster Management",
                vec![
                    dashboard("Nodes", &["cluster"], false, None),
                    dashboard("Capacity", &["cluster", "resources"], false, None),
                ],
            ),
            folder(
                "Jobs",
                vec![dashboard("Job Details", &["jobs"], true, None)],
            ),
            folder(
                "Services",
                vec![dashboard("LLM Endpoint", &["services", "llm"], false, None)],
            ),
            folder(
                "Slurm",
                vec![
                    dashboard(
                        "Current Jobs",
                        &["slurm", "jobs"],
                        true,
                        Some("/slurm/current-jobs"),
                    ),
                    dashboard("Job Archive", &["slurm", "jobs"], false, None),
                    dashboard("Job Statistics", &["slurm", "stats"], false, None),
                    dashboard("Job Wait Times", &["slurm", "stats"], false, None),
                    dashboard("Resources", &["slurm", "resources"], false, None),
                    dashboard("Resources Playground", &["slurm", "resources", "wip"], false, None),
                ],
            ),
        ],
    }
}

fn seed_jobs() -> Vec<JobSummary> {
    // (id, user, name, state, partition, nodes, runtime_s, limit_s, gpus, cpus,
    //  mem_mib, gpu%, cpu%, mem%, reason)
    let rows: &[(
        i64,
        &str,
        &str,
        &str,
        &str,
        Option<&str>,
        Option<f64>,
        f64,
        i64,
        i64,
        i64,
        Option<f32>,
        Option<f32>,
        Option<f32>,
        Option<&str>,
    )] = &[
        (3341149, "shatskiy", "octic_toy", "RUNNING", "A100-RP", Some("virt-3326"), Some(8_613.0), 57_600.0, 1, 12, 262_144, Some(95.8), Some(87.7), Some(11.8), None),
        (3341592, "mueller", "train_bert", "RUNNING", "H200-AV", Some("virt-3401"), Some(41_200.0), 86_400.0, 4, 48, 786_432, Some(72.4), Some(63.1), Some(48.0), None),
        (3341601, "chen", "sd_finetune", "RUNNING", "L40S-AV", Some("virt-3512"), Some(2_950.0), 21_600.0, 2, 24, 393_216, Some(41.2), Some(28.9), Some(33.5), None),
        (3341710, "shatskiy", "octic_eval_knn", "PENDING", "H200-AV,L40S-AV,A100-RP", None, None, 10_800.0, 1, 12, 204_800, None, None, None, Some("Priority")),
        (3341711, "shatskiy", "octic_eval_knn", "PENDING", "H200-AV,L40S-AV,A100-RP", None, None, 10_800.0, 1, 12, 204_800, None, None, None, Some("Dependency")),
        (3341712, "okafor", "hpo_sweep", "PENDING", "A100-RP", None, None, 43_200.0, 8, 96, 1_048_576, None, None, None, Some("Resources")),
        (3341713, "novak", "big_infer", "PENDING", "RTXA6000-SLT", None, None, 21_600.0, 2, 16, 262_144, None, None, None, Some("(uid_4956_not_in_group_permitted_to_use_this_partition_(RTXA6000-SLT)._groups_allowed:_slt)")),
        (3340880, "mueller", "prep_data", "COMPLETED", "cpu", Some("cn-14"), Some(1_320.0), 3_600.0, 0, 8, 65_536, None, Some(54.0), Some(22.1), None),
        (3340921, "chen", "export_ckpt", "COMPLETED", "cpu", Some("cn-09"), Some(410.0), 1_800.0, 0, 4, 32_768, None, Some(31.7), Some(9.8), None),
        (3340755, "okafor", "train_gnn", "FAILED", "A100-RP", Some("virt-3330"), Some(612.0), 28_800.0, 2, 24, 393_216, Some(3.1), Some(12.0), Some(88.9), None),
        (3340310, "patel", "megatron_run", "TIMEOUT", "H200-AV", Some("virt-3410"), Some(86_400.0), 86_400.0, 8, 96, 1_572_864, Some(81.0), Some(70.2), Some(60.4), None),
        (3340120, "patel", "debug_shell", "CANCELLED", "A100-RP", Some("virt-3327"), Some(180.0), 7_200.0, 1, 12, 131_072, None, None, None, None),
        (3341150, "chen", "notebook", "SUSPENDED", "L40S-AV", Some("virt-3515"), Some(5_400.0), 14_400.0, 1, 12, 196_608, Some(0.0), Some(2.4), Some(41.0), None),
    ];

    rows.iter()
        .map(|r| JobSummary {
            id: r.0,
            array_job_id: None,
            array_task_id: None,
            name: Some(r.2.to_string()),
            user: r.1.to_string(),
            user_raw: format!("{}(1000)", r.1),
            group: Some("staff".to_string()),
            account: Some("research".to_string()),
            state: JobState::parse(r.3),
            state_raw: r.3.to_string(),
            reason: r.14.map(|s| s.to_string()),
            partition: Some(r.4.to_string()),
            qos: Some("normal".to_string()),
            nodelist: r.5.map(|s| s.to_string()),
            submit_time: Some(1_756_000_000),
            start_time: r.6.map(|_| 1_756_010_000),
            end_time: None,
            runtime_secs: r.6,
            time_limit_secs: Some(r.7),
            num_nodes: Some(1),
            num_cpus: r.9,
            num_gpus: Some(r.8),
            mem_mib: Some(r.10),
            gpu_util: r.11,
            cpu_util: r.12,
            mem_util: r.13,
            exit_code: None,
        })
        .collect()
}
