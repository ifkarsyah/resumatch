use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ── Job structs ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct Job {
    title: String,
    description: Option<String>,
    requirements: Option<Vec<String>>,
    responsibilities: Option<Vec<String>>,
    tools: Option<Vec<String>>,
}

// ── Resume structs ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct Resume {
    experience: Option<Vec<Experience>>,
    skills: Option<Skills>,
    projects: Option<Vec<Project>>,
}

#[derive(Deserialize)]
struct Experience {
    bullets: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct Skills {
    tools: Option<Vec<String>>,
    languages: Option<Vec<String>>,
    frameworks: Option<Vec<String>>,
    databases: Option<Vec<String>>,
    cloud: Option<Vec<String>>,
    other: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct Project {
    description: Option<String>,
    technologies: Option<Vec<String>>,
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Normalise a token for comparison: lowercase, trim punctuation.
fn normalise(s: &str) -> String {
    s.trim()
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

/// Collect every word/token from a block of text.
fn tokens(text: &str) -> HashSet<String> {
    text.split_whitespace()
        .map(|w| normalise(w))
        .filter(|w| !w.is_empty())
        .collect()
}

/// Concatenate all resume text into one big string for keyword scanning.
fn resume_full_text(resume: &Resume) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(experiences) = &resume.experience {
        for exp in experiences {
            if let Some(bullets) = &exp.bullets {
                parts.extend(bullets.iter().cloned());
            }
        }
    }

    if let Some(projects) = &resume.projects {
        for proj in projects {
            if let Some(desc) = &proj.description {
                parts.push(desc.clone());
            }
            if let Some(techs) = &proj.technologies {
                parts.extend(techs.iter().cloned());
            }
        }
    }

    if let Some(skills) = &resume.skills {
        for list in [
            &skills.tools,
            &skills.languages,
            &skills.frameworks,
            &skills.databases,
            &skills.cloud,
            &skills.other,
        ] {
            if let Some(items) = list {
                parts.extend(items.iter().cloned());
            }
        }
    }

    parts.join(" ")
}

/// Collect explicit skill tokens from resume (skills section + project techs).
fn resume_explicit_tools(resume: &Resume) -> HashSet<String> {
    let mut set: HashSet<String> = HashSet::new();

    if let Some(skills) = &resume.skills {
        for list in [
            &skills.tools,
            &skills.languages,
            &skills.frameworks,
            &skills.databases,
            &skills.cloud,
            &skills.other,
        ] {
            if let Some(items) = list {
                for item in items {
                    set.insert(normalise(item));
                }
            }
        }
    }

    if let Some(projects) = &resume.projects {
        for proj in projects {
            if let Some(techs) = &proj.technologies {
                for tech in techs {
                    set.insert(normalise(tech));
                }
            }
        }
    }

    set
}

/// All text from the job posting flattened.
fn job_full_text(job: &Job) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(desc) = &job.description {
        parts.push(desc.clone());
    }
    if let Some(reqs) = &job.requirements {
        parts.extend(reqs.iter().cloned());
    }
    if let Some(resps) = &job.responsibilities {
        parts.extend(resps.iter().cloned());
    }
    parts.join(" ")
}

// ── Analysis ─────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct AnalysisResult {
    missing_tools: Vec<String>,
    found_tools: Vec<String>,
    relevance_score: f64,
    matched_keywords: Vec<String>,
    total_job_keywords: usize,
}

fn analyse(job: &Job, resume: &Resume) -> AnalysisResult {
    let job_tools: Vec<String> = job
        .tools
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|t| t.clone())
        .collect();

    let resume_text = resume_full_text(resume);
    let resume_tokens = tokens(&resume_text);
    let resume_explicit = resume_explicit_tools(resume);

    // ── 1. Missing / found tools ─────────────────────────────────────────────
    // A tool is "found" if its normalised form appears anywhere in resume text.
    let mut missing_tools: Vec<String> = Vec::new();
    let mut found_tools: Vec<String> = Vec::new();

    for tool in &job_tools {
        let norm = normalise(tool);
        if resume_tokens.contains(&norm) || resume_explicit.contains(&norm) {
            found_tools.push(tool.clone());
        } else {
            missing_tools.push(tool.clone());
        }
    }

    // ── 2. Relevance score ───────────────────────────────────────────────────
    // Keyword overlap: meaningful words from job text that appear in resume.
    // We skip very short / common stop-words.
    let stop_words: HashSet<&str> = [
        "a", "an", "the", "and", "or", "of", "to", "in", "for", "with",
        "on", "at", "by", "is", "are", "be", "as", "it", "its", "we",
        "our", "you", "your", "they", "their", "that", "this", "from",
        "will", "can", "may", "has", "have", "had", "was", "were", "do",
        "does", "not", "all", "any", "more", "also", "into", "than",
        "how", "what", "who", "which", "while", "about", "up", "out",
        "other", "new", "per",
    ]
    .iter()
    .cloned()
    .collect();

    let job_text = job_full_text(job);
    let job_keywords: HashSet<String> = tokens(&job_text)
        .into_iter()
        .filter(|w| w.len() > 3 && !stop_words.contains(w.as_str()))
        .collect();

    let matched: Vec<String> = job_keywords
        .iter()
        .filter(|kw| resume_tokens.contains(*kw))
        .cloned()
        .collect();

    let total = job_keywords.len();
    let score = if total == 0 {
        0.0
    } else {
        matched.len() as f64 / total as f64 * 100.0
    };

    let mut matched_sorted = matched;
    matched_sorted.sort();

    AnalysisResult {
        missing_tools,
        found_tools,
        relevance_score: score,
        matched_keywords: matched_sorted,
        total_job_keywords: total,
    }
}

// ── Main ─────────────────────────────────────────────────────────────────────

const JOB_PATH: &str = "data/input/job.json";
const RESUME_PATH: &str = "data/input/resume.json";
const OUTPUT_PATH: &str = "data/output/result.json";

fn main() -> Result<()> {
    let job_str = std::fs::read_to_string(JOB_PATH)
        .with_context(|| format!("Cannot read job file: {JOB_PATH}"))?;
    let resume_str = std::fs::read_to_string(RESUME_PATH)
        .with_context(|| format!("Cannot read resume file: {RESUME_PATH}"))?;

    let job: Job = serde_json::from_str(&job_str).context("Failed to parse job.json")?;
    let resume: Resume =
        serde_json::from_str(&resume_str).context("Failed to parse resume.json")?;

    let result = analyse(&job, &resume);

    // ── Write JSON output ─────────────────────────────────────────────────────
    std::fs::create_dir_all("data/output").context("Cannot create data/output dir")?;
    let json_out = serde_json::to_string_pretty(&result).context("Failed to serialise result")?;
    std::fs::write(OUTPUT_PATH, &json_out)
        .with_context(|| format!("Cannot write output file: {OUTPUT_PATH}"))?;

    // ── Print report ─────────────────────────────────────────────────────────
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║              RESUME MATCH REPORT                        ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    println!("  Job title : {}", job.title);
    println!();

    // Relevance score
    let bar_len = 40usize;
    let filled = (result.relevance_score / 100.0 * bar_len as f64).round() as usize;
    let bar: String = format!(
        "[{}{}]",
        "█".repeat(filled),
        "░".repeat(bar_len - filled)
    );
    println!(
        "  Relevance Score : {:.1}%  {}",
        result.relevance_score, bar
    );
    println!(
        "  Matched {} / {} job keywords",
        result.matched_keywords.len(),
        result.total_job_keywords
    );
    println!();

    // Tools breakdown
    println!("─────────────────────────────────────────────────────────────");
    println!("  TOOLS IN JOB POSTING");
    println!("─────────────────────────────────────────────────────────────");
    if result.found_tools.is_empty() && result.missing_tools.is_empty() {
        println!("  (no tools listed in job.json)");
    } else {
        for tool in &result.found_tools {
            println!("  ✓  {}", tool);
        }
        for tool in &result.missing_tools {
            println!("  ✗  {}", tool);
        }
    }
    println!();
    println!("  Found on resume : {}", result.found_tools.len());
    println!("  Missing         : {}", result.missing_tools.len());

    if !result.missing_tools.is_empty() {
        println!();
        println!("─────────────────────────────────────────────────────────────");
        println!("  MISSING TOOLS  (consider adding or highlighting these)");
        println!("─────────────────────────────────────────────────────────────");
        for tool in &result.missing_tools {
            println!("  • {}", tool);
        }
    }

    println!();
    println!("─────────────────────────────────────────────────────────────");
    println!("  MATCHED KEYWORDS (sample, alphabetical)");
    println!("─────────────────────────────────────────────────────────────");
    let sample: Vec<&String> = result.matched_keywords.iter().take(30).collect();
    for kw in &sample {
        print!("  {kw}");
    }
    if result.matched_keywords.len() > 30 {
        print!(
            "  … (+{} more)",
            result.matched_keywords.len() - 30
        );
    }
    println!();
    println!();
    println!("  Output written to: {OUTPUT_PATH}");
    println!();

    Ok(())
}
