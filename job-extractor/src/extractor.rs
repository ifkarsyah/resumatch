use anyhow::Result;

use crate::lever::{fetch_posting, parse_lever_url, parse_list_items};
use crate::models::JobPosting;
use crate::tools::extract_tools;

pub async fn extract_job(client: &reqwest::Client, url: &str) -> Result<JobPosting> {
    let (company, job_id) = parse_lever_url(url)?;
    let posting = fetch_posting(client, &company, &job_id).await?;

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

    for list in posting.lists.unwrap_or_default() {
        let heading = list.text.to_lowercase();
        let items = parse_list_items(&list.content);
        if heading.contains("responsibilit") {
            responsibilities = items;
        } else if heading.contains("requirement") || heading.contains("qualif") {
            requirements = items;
        }
    }

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
