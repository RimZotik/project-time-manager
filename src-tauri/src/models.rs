use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppPayload {
    pub tracker: TrackerPayload,
    pub projects: Vec<ProjectSummary>,
    pub selected_project: Option<ProjectRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrackerPayload {
    pub status: String,
    pub active_project_id: Option<String>,
    pub running_since: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub client: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectRecord {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub client: String,
    pub note: String,
    pub created_at: String,
    pub updated_at: String,
    pub sessions: Vec<SessionRecord>,
    pub apps: Vec<AppUsageRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppUsageRecord {
    pub key: String,
    pub name: String,
    pub process_name: String,
    #[serde(default)]
    pub process_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_data_url: Option<String>,
    pub kind: String,
    pub enabled: bool,
    pub time_seconds: u64,
    pub tabs: Vec<TabUsageRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TabUsageRecord {
    pub key: String,
    pub title: String,
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub favicon_url: Option<String>,
    pub enabled: bool,
    pub time_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionRecord {
    pub id: String,
    pub started_at: String,
    pub stopped_at: Option<String>,
    pub duration_seconds: u64,
    pub app_count: usize,
    pub browser_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct WindowObservation {
    pub process_name: String,
    pub process_path: String,
    pub icon_data_url: Option<String>,
    pub window_title: String,
    pub browser_name: Option<String>,
    pub tab_title: Option<String>,
    pub url: Option<String>,
    pub favicon_url: Option<String>,
}

impl ProjectRecord {
    pub fn summary(&self) -> ProjectSummary {
        ProjectSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            client: self.client.clone(),
            updated_at: self.updated_at.clone(),
        }
    }
}
