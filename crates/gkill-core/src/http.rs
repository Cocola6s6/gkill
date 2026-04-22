use anyhow::{bail, Result};
use bytes::Bytes;
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("HTTP {status}: {message}")]
pub struct HttpError {
    pub status: u16,
    pub message: String,
}

pub struct Client {
    inner: reqwest::Client,
    pub token: Option<String>,
    pub registry: String,
}

impl Client {
    pub fn new(registry: &str, token: Option<String>) -> Self {
        let inner = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            inner,
            token,
            registry: registry.trim_end_matches('/').to_string(),
        }
    }

    fn auth_header(&self) -> Option<String> {
        self.token
            .as_ref()
            .map(|t| format!("Bearer {}", t.trim()))
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.registry, path)
    }

    async fn check_response(res: reqwest::Response) -> Result<reqwest::Response> {
        let status = res.status();
        if !status.is_success() {
            let code = status.as_u16();
            let body = res.text().await.unwrap_or_default();
            if code == 403 {
                bail!(HttpError {
                    status: code,
                    message: format!("无安装权限（403）。{}", body),
                });
            }
            bail!(HttpError {
                status: code,
                message: body,
            });
        }
        Ok(res)
    }

    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let mut req = self.inner.get(self.url(path));
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        let res = req.send().await?;
        let res = Self::check_response(res).await?;
        // SkillHub wraps responses: { "code": 0, "data": {...} }
        let raw: serde_json::Value = res.json().await?;
        if let Some(data) = raw.get("data") {
            Ok(serde_json::from_value(data.clone())?)
        } else {
            Ok(serde_json::from_value(raw)?)
        }
    }

    pub async fn get_bytes(&self, path: &str) -> Result<Bytes> {
        let mut req = self.inner.get(self.url(path));
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        let res = req.send().await?;
        let res = Self::check_response(res).await?;
        Ok(res.bytes().await?)
    }

    pub async fn post_multipart(
        &self,
        path: &str,
        form: reqwest::multipart::Form,
    ) -> Result<serde_json::Value> {
        let mut req = self.inner.post(self.url(path)).multipart(form);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        let res = req.send().await?;
        let res = Self::check_response(res).await?;
        Ok(res.json().await?)
    }
}
