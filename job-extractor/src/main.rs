use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

// --- Output schema ---

#[derive(Serialize)]
struct JobPosting {
    url: String,
    title: String,
    country: String,
    location: String,
    created_at: u64,
    salary_range: Option<SalaryRange>,
    description: String,
    requirements: Vec<String>,
    responsibilities: Vec<String>,
    tools: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct SalaryRange {
    min: Option<f64>,
    max: Option<f64>,
    currency: Option<String>,
    interval: Option<String>,
}

// --- Lever API response types ---

#[derive(Deserialize)]
struct LeverCategories {
    location: Option<String>,
}

#[derive(Deserialize)]
struct LeverPosting {
    text: String,
    country: Option<String>,
    categories: Option<LeverCategories>,
    #[serde(rename = "createdAt")]
    created_at: Option<u64>,
    #[serde(rename = "salaryRange")]
    salary_range: Option<SalaryRange>,
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

// --- Tool detection ---

const KNOWN_TOOLS: &[&str] = &[
    "Kafka", "Spark", "Flink", "Python", "Scala", "Java", "Go", "Rust",
    "SQL", "PostgreSQL", "MySQL", "MongoDB", "Redis", "Elasticsearch",
    "Snowflake", "dbt", "Airflow", "Kubernetes", "Docker", "Terraform",
    "AWS", "GCP", "Azure", "Hadoop", "Hive", "Trino", "Presto",
    "Databricks", "Redshift", "BigQuery", "Kinesis", "Pulsar",
];

fn extract_tools(text: &str) -> Vec<String> {
    KNOWN_TOOLS
        .iter()
        .filter(|&&tool| {
            // case-insensitive whole-word match using char boundaries
            let lower_text = text.to_lowercase();
            let lower_tool = tool.to_lowercase();
            let mut start = 0;
            while let Some(pos) = lower_text[start..].find(&lower_tool) {
                let abs = start + pos;
                let before_ok = abs == 0
                    || !lower_text[..abs]
                        .chars()
                        .last()
                        .map(|c| c.is_alphanumeric())
                        .unwrap_or(false);
                let after_ok = abs + lower_tool.len() >= lower_text.len()
                    || !lower_text[abs + lower_tool.len()..]
                        .chars()
                        .next()
                        .map(|c| c.is_alphanumeric())
                        .unwrap_or(false);
                if before_ok && after_ok {
                    return true;
                }
                start = abs + 1;
            }
            false
        })
        .map(|&t| t.to_string())
        .collect()
}

// --- Main extraction ---

async fn extract_job(client: &reqwest::Client, url: &str) -> Result<JobPosting> {
    let (company, job_id) = parse_lever_url(url)?;

    let api_url = format!("https://api.lever.co/v0/postings/{}/{}", company, job_id);
    let posting: LeverPosting = client
        .get(&api_url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let title = posting.text;
    let country = posting.country.unwrap_or_default();
    let location = posting
        .categories
        .and_then(|c| c.location)
        .unwrap_or_default();
    let created_at = posting.created_at.unwrap_or(0);
    let salary_range = posting.salary_range;
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

    // Scan all text content for known tools
    let all_text = format!(
        "{} {} {} {}",
        title,
        description,
        requirements.join(" "),
        responsibilities.join(" ")
    );
    let tools = extract_tools(&all_text);

    Ok(JobPosting {
        url: url.to_string(),
        title,
        country,
        location,
        created_at,
        salary_range,
        description,
        requirements,
        responsibilities,
        tools,
    })
}

#[tokio::main]
async fn main() {
    let input_path = Path::new("input.txt");
    let output_path = Path::new("output.json");

    if !input_path.exists() {
        eprintln!("Error: input.txt not found");
        std::process::exit(1);
    }

    let content = std::fs::read_to_string(input_path).expect("Failed to read input.txt");
    let urls: Vec<&str> = content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    if urls.is_empty() {
        eprintln!("No URLs found in input.txt");
        std::process::exit(1);
    }

    println!("Extracting {} job(s)...", urls.len());

    let client = reqwest::Client::new();
    let mut results = Vec::new();

    for url in &urls {
        print!("  {} ... ", url);
        match extract_job(&client, url).await {
            Ok(job) => {
                println!("ok ({})", job.title);
                results.push(serde_json::to_value(job).unwrap());
            }
            Err(e) => {
                println!("error: {}", e);
                results.push(serde_json::json!({ "url": url, "error": e.to_string() }));
            }
        }
    }

    let output = serde_json::to_string_pretty(&results).unwrap();
    std::fs::write(output_path, &output).expect("Failed to write output.json");
    println!("\nWrote {} result(s) to output.json", results.len());
}
