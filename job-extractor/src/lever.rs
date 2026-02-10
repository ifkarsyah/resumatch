use anyhow::{anyhow, Result};
use serde::Deserialize;

use crate::models::SalaryRange;

// --- Lever API response types ---

#[derive(Deserialize)]
pub struct LeverCategories {
    pub location: Option<String>,
}

#[derive(Deserialize)]
pub struct LeverPosting {
    pub text: String,
    pub country: Option<String>,
    pub categories: Option<LeverCategories>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<u64>,
    #[serde(rename = "salaryRange")]
    pub salary_range: Option<SalaryRange>,
    #[serde(rename = "descriptionPlain")]
    pub description_plain: Option<String>,
    pub lists: Option<Vec<LeverList>>,
}

#[derive(Deserialize)]
pub struct LeverList {
    pub text: String,
    pub content: String,
}

// --- URL parsing ---

pub fn parse_lever_url(url: &str) -> Result<(String, String)> {
    // Expected: https://jobs.lever.co/{company}/{job-id}
    let url = url.trim_end_matches('/');
    let parts: Vec<&str> = url.split('/').collect();
    let lever_idx = parts
        .iter()
        .position(|&p| p == "jobs.lever.co")
        .ok_or_else(|| anyhow!("URL is not a Lever job posting (expected jobs.lever.co)"))?;

    let company = parts
        .get(lever_idx + 1)
        .ok_or_else(|| anyhow!("Missing company in Lever URL"))?;
    let job_id = parts
        .get(lever_idx + 2)
        .ok_or_else(|| anyhow!("Missing job ID in Lever URL"))?;

    Ok((company.to_string(), job_id.to_string()))
}

// --- API fetch ---

pub async fn fetch_posting(
    client: &reqwest::Client,
    company: &str,
    job_id: &str,
) -> Result<LeverPosting> {
    let url = format!("https://api.lever.co/v0/postings/{}/{}", company, job_id);
    let posting = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(posting)
}

// --- HTML parsing helpers ---

pub fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

pub fn parse_list_items(html: &str) -> Vec<String> {
    // Content looks like: <li>item one</li><li>item two</li>
    html.split("<li>")
        .skip(1)
        .map(|chunk| {
            let raw = chunk.split("</li>").next().unwrap_or("");
            decode_html_entities(raw)
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|s| !s.is_empty())
        .collect()
}
