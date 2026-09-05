//! The dashboard catalog shown on the home screen: a list of folders, each
//! holding a list of dashboards.

use serde::{Deserialize, Serialize};

/// The complete set of folders and dashboards available to the viewer.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Catalog {
    pub folders: Vec<Folder>,
}

/// A named group of dashboards.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Folder {
    /// Stable identifier, used as a list key.
    pub id: String,
    pub name: String,
    pub dashboards: Vec<Dashboard>,
}

/// A single dashboard entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: String,
    pub title: String,
    /// URL-friendly identifier for the (future) `/d/{slug}` route.
    pub slug: String,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub starred: bool,
    /// Explicit destination for dashboards that already have a real screen.
    /// `None` ⇒ fall back to the placeholder `/d/{slug}` route.
    #[serde(default)]
    pub route: Option<String>,
}

impl Catalog {
    /// All distinct tags across every dashboard, sorted alphabetically.
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .folders
            .iter()
            .flat_map(|f| &f.dashboards)
            .flat_map(|d| d.tags.iter().cloned())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    /// Return the folders that match a free-text `query` and carry every tag in
    /// `tags` (tags are an AND filter). Folders whose name matches the query
    /// keep all of their (tag-filtered) dashboards; other folders keep only the
    /// dashboards that match. Folders left with no dashboards are dropped.
    pub fn filter(&self, query: &str, tags: &[String]) -> Vec<Folder> {
        let q = query.trim().to_lowercase();

        self.folders
            .iter()
            .filter_map(|folder| {
                let folder_matches = !q.is_empty() && folder.name.to_lowercase().contains(&q);

                let dashboards: Vec<Dashboard> = folder
                    .dashboards
                    .iter()
                    .filter(|d| d.matches_text(&q, folder_matches) && d.has_all_tags(tags))
                    .cloned()
                    .collect();

                if dashboards.is_empty() {
                    None
                } else {
                    Some(Folder {
                        dashboards,
                        ..folder.clone()
                    })
                }
            })
            .collect()
    }
}

impl Dashboard {
    fn matches_text(&self, query_lower: &str, folder_matches: bool) -> bool {
        if query_lower.is_empty() || folder_matches {
            return true;
        }
        self.title.to_lowercase().contains(query_lower)
            || self.tags.iter().any(|t| t.to_lowercase().contains(query_lower))
    }

    fn has_all_tags(&self, wanted: &[String]) -> bool {
        wanted.iter().all(|w| self.tags.iter().any(|t| t == w))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dash(id: &str, title: &str, tags: &[&str]) -> Dashboard {
        Dashboard {
            id: id.into(),
            title: title.into(),
            slug: id.into(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            description: None,
            starred: false,
            route: None,
        }
    }

    fn sample() -> Catalog {
        Catalog {
            folders: vec![
                Folder {
                    id: "jobs".into(),
                    name: "Jobs".into(),
                    dashboards: vec![dash("job-details", "Job Details", &["jobs"])],
                },
                Folder {
                    id: "slurm".into(),
                    name: "Slurm".into(),
                    dashboards: vec![
                        dash("current-jobs", "Current Jobs", &["slurm", "jobs"]),
                        dash("resources", "Resources", &["slurm", "resources"]),
                    ],
                },
            ],
        }
    }

    #[test]
    fn all_tags_are_sorted_and_unique() {
        assert_eq!(sample().all_tags(), vec!["jobs", "resources", "slurm"]);
    }

    #[test]
    fn text_query_matches_title() {
        let out = sample().filter("current", &[]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].dashboards.len(), 1);
        assert_eq!(out[0].dashboards[0].id, "current-jobs");
    }

    #[test]
    fn folder_name_match_keeps_all_children() {
        let out = sample().filter("slurm", &[]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].dashboards.len(), 2);
    }

    #[test]
    fn tag_filter_is_conjunctive() {
        let both = vec!["slurm".to_string(), "jobs".to_string()];
        let out = sample().filter("", &both);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].dashboards[0].id, "current-jobs");
    }

    #[test]
    fn no_match_yields_no_folders() {
        assert!(sample().filter("nonsense", &[]).is_empty());
    }
}
