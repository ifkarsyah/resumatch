use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Resume {
    pub basics: Basics,
    pub summary: Option<String>,
    pub experience: Vec<Experience>,
    pub education: Vec<Education>,
    pub skills: Skills,
    pub projects: Vec<Project>,
    pub achievements: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Basics {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Experience {
    pub company: String,
    pub title: String,
    pub location: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub bullets: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Education {
    pub institution: String,
    pub degree: Option<String>,
    pub field: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub gpa: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Skills {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub databases: Vec<String>,
    pub tools: Vec<String>,
    pub cloud: Vec<String>,
    pub other: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub description: Option<String>,
    pub technologies: Vec<String>,
}
