use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRef {
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDetail {
    pub published_version: Option<VersionRef>,
    pub headline_version: Option<VersionRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionDetail {
    pub published_at: String,
    pub parsed_metadata_json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillItem {
    pub slug: String,
    pub display_name: Option<String>,
    pub namespace: Option<String>,
    pub summary: Option<String>,
    pub download_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPage {
    pub items: Vec<SkillItem>,
    pub total: u64,
    pub size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillMeta {
    pub version: String,
    pub name: String,
    pub namespace: String,
    pub published_at: String,
}
