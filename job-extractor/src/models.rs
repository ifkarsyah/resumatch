use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct JobPosting {
    pub url: String,
    pub title: String,
    pub country: String,
    pub location: String,
    pub created_at: u64,
    pub salary_range: Option<SalaryRange>,
    pub description: String,
    pub requirements: Vec<String>,
    pub responsibilities: Vec<String>,
    pub tools: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SalaryRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub currency: Option<String>,
    pub interval: Option<String>,
}
