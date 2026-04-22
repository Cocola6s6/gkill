use crate::http::Client;
use crate::types::{SkillDetail, SkillPage, VersionDetail};
use anyhow::Result;
use bytes::Bytes;

pub async fn get_skill(client: &Client, ns: &str, slug: &str) -> Result<SkillDetail> {
    client
        .get_json(&format!("/api/v1/skills/{}/{}", ns, slug))
        .await
}

pub async fn get_version(
    client: &Client,
    ns: &str,
    slug: &str,
    version: &str,
) -> Result<VersionDetail> {
    client
        .get_json(&format!("/api/v1/skills/{}/{}/versions/{}", ns, slug, version))
        .await
}

pub async fn download_skill(
    client: &Client,
    ns: &str,
    slug: &str,
    version: &str,
) -> Result<Bytes> {
    client
        .get_bytes(&format!(
            "/api/v1/skills/{}/{}/versions/{}/download",
            ns, slug, version
        ))
        .await
}

pub async fn search_skills(
    client: &Client,
    q: &str,
    sort: &str,
    page: u32,
    size: u32,
) -> Result<SkillPage> {
    let path = format!(
        "/api/web/skills?q={}&sort={}&page={}&size={}",
        urlencoding(q),
        sort,
        page,
        size
    );
    client.get_json(&path).await
}

pub async fn whoami(client: &Client) -> Result<serde_json::Value> {
    client.get_json("/api/v1/whoami").await
}

pub async fn my_namespaces(client: &Client) -> Result<serde_json::Value> {
    client.get_json("/api/v1/me/namespaces").await
}

pub async fn publish_skill(
    client: &Client,
    ns: &str,
    visibility: &str,
    archive: Bytes,
    slug: &str,
) -> Result<()> {
    let part = reqwest::multipart::Part::bytes(archive.to_vec())
        .file_name(format!("{}.zip", slug))
        .mime_str("application/zip")?;
    let form = reqwest::multipart::Form::new()
        .text("visibility", visibility.to_string())
        .part("file", part);
    client
        .post_multipart(&format!("/api/v1/skills/{}/publish", ns), form)
        .await?;
    Ok(())
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c.is_alphanumeric() || "-_.~".contains(c) {
                vec![c]
            } else {
                format!("%{:02X}", c as u32).chars().collect()
            }
        })
        .collect()
}
