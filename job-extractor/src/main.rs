use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

// --- Output schema ---

#[derive(Serialize)]
struct JobPosting {
    title: String,
    location: String,
    description: String,
    requirements: Vec<String>,
    responsibilities: Vec<String>,
}

// --- Lever API response types ---

#[derive(Deserialize)]
struct LeverCategories {
    location: Option<String>,
}

#[derive(Deserialize)]
struct LeverPosting {
    text: String,
    categories: Option<LeverCategories>,
    #[serde(rename = "descriptionPlain")]
    description_plain: Option<String>,
    lists: Option<Vec<LeverList>>,
}

#[derive(Deserialize)]
struct LeverList {
    text: String,
    content: String,
}

// --- URL parsing ---

fn parse_lever_url(url: &str) -> Result<(String, String)> {
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

// --- HTML list content → Vec<String> ---

fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn parse_list_items(html: &str) -> Vec<String> {
    // Content looks like: <li>item one</li><li>item two</li>
    html.split("<li>")
        .skip(1) // first element before any <li> is empty
        .map(|chunk| {
            let raw = chunk.split("</li>").next().unwrap_or("");
            decode_html_entities(raw).split_whitespace().collect::<Vec<_>>().join(" ")
        })
        .filter(|s| !s.is_empty())
        .collect()
}

// --- Main extraction ---

async fn extract_job(url: &str) -> Result<JobPosting> {
    let (company, job_id) = parse_lever_url(url)?;

    let api_url = format!("https://api.lever.co/v0/postings/{}/{}", company, job_id);
    let client = reqwest::Client::new();
    let posting: LeverPosting = client
        .get(&api_url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let title = posting.text;
    let location = posting
        .categories
        .and_then(|c| c.location)
        .unwrap_or_default();
    let description = posting.description_plain.unwrap_or_default();

    let mut requirements = Vec::new();
    let mut responsibilities = Vec::new();

    if let Some(lists) = posting.lists {
        for list in lists {
            let heading = list.text.to_lowercase();
            let items = parse_list_items(&list.content);
            if heading.contains("responsibilit") {
                responsibilities = items;
            } else if heading.contains("requirement") || heading.contains("qualif") {
                requirements = items;
            }
        }
    }

    Ok(JobPosting {
        title,
        location,
        description,
        requirements,
        responsibilities,
    })
}

#[tokio::main]
async fn main() {
    let url = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: job-extractor <lever-job-url>");
        std::process::exit(1);
    });

    match extract_job(&url).await {
        Ok(job) => println!("{}", serde_json::to_string_pretty(&job).unwrap()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
