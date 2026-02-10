use regex::Regex;
use crate::model::*;

pub fn parse_resume(text: &str) -> Resume {
    let lines: Vec<&str> = text.lines().collect();

    Resume {
        basics: parse_basics(&lines),
        summary: parse_summary(&lines),
        experience: parse_experience(&lines),
        education: parse_education(&lines),
        skills: parse_skills(&lines),
        projects: parse_projects(&lines),
        achievements: parse_achievements(&lines),
    }
}

// Section headers used in this resume (and common variants)
const ALL_SECTION_HEADERS: &[&str] = &[
    "EXPERIENCES", "EXPERIENCE", "WORK EXPERIENCE", "PROFESSIONAL EXPERIENCE",
    "EDUCATION", "EDUCATIONAL BACKGROUND",
    "SKILLS", "TECHNICAL SKILLS", "TECHNOLOGIES",
    "PROJECTS", "PERSONAL PROJECTS",
    "ACHIEVEMENTS", "CERTIFICATIONS", "CERTIFICATES", "AWARDS",
    "SUMMARY", "OBJECTIVE", "PROFILE",
];

/// Returns (start, end) line indices (exclusive of the header line itself)
fn section_range(lines: &[&str], names: &[&str]) -> Option<(usize, usize)> {
    let start_idx = lines.iter().position(|l| {
        let upper = l.trim().to_uppercase();
        names.iter().any(|s| upper == *s)
    })?;

    let end_idx = lines[start_idx + 1..]
        .iter()
        .position(|l| {
            let upper = l.trim().to_uppercase();
            ALL_SECTION_HEADERS.iter().any(|s| upper == *s)
        })
        .map(|i| start_idx + 1 + i)
        .unwrap_or(lines.len());

    Some((start_idx + 1, end_idx))
}

fn parse_basics(lines: &[&str]) -> Basics {
    let email_re = Regex::new(r"[\w.+-]+@[\w.-]+\.\w+").unwrap();
    let phone_re = Regex::new(r"[\+]?[\d\s\-\(\)]{8,20}").unwrap();
    let linkedin_re = Regex::new(r"linkedin\.com/in/([\w\-]+)").unwrap();
    let github_re = Regex::new(r"github\.com/([\w\-]+)").unwrap();
    let website_re = Regex::new(r"([\w\-]+\.github\.io[\w/]*)").unwrap();
    // Location: look for "City, Country" pattern in the phone/location line
    let location_re = Regex::new(r"([A-Z][a-z]+,\s*[A-Za-z ]+)").unwrap();

    let mut basics = Basics {
        name: None,
        email: None,
        phone: None,
        location: None,
        linkedin: None,
        github: None,
        website: None,
    };

    // Name: first non-empty, all-caps or title-case short line
    for line in lines.iter().take(10) {
        let t = line.trim();
        if t.is_empty() { continue; }
        if t.len() > 3 && t.len() < 60 && !email_re.is_match(t) && !t.contains('@') {
            basics.name = Some(t.to_string());
            break;
        }
    }

    // Contact info: scan first 15 lines
    for line in lines.iter().take(15) {
        let t = line.trim();
        if t.is_empty() { continue; }

        if basics.email.is_none() {
            if let Some(m) = email_re.find(t) {
                basics.email = Some(m.as_str().to_string());
            }
        }
        if basics.phone.is_none() {
            if let Some(m) = phone_re.find(t) {
                let p = m.as_str().trim().to_string();
                if p.chars().filter(|c| c.is_ascii_digit()).count() >= 7 {
                    basics.phone = Some(p);
                }
            }
        }
        if basics.location.is_none() {
            if let Some(m) = location_re.find(t) {
                basics.location = Some(m.as_str().trim().to_string());
            }
        }
        if basics.linkedin.is_none() {
            if let Some(m) = linkedin_re.captures(t) {
                basics.linkedin = Some(format!("https://linkedin.com/in/{}", &m[1]));
            }
        }
        if basics.github.is_none() {
            if let Some(m) = github_re.captures(t) {
                basics.github = Some(format!("https://github.com/{}", &m[1]));
            }
        }
        if basics.website.is_none() {
            if let Some(m) = website_re.find(t) {
                basics.website = Some(format!("https://{}", m.as_str()));
            }
        }
    }

    basics
}

fn parse_summary(lines: &[&str]) -> Option<String> {
    let (start, end) = section_range(lines, &["SUMMARY", "OBJECTIVE", "PROFILE"])?;
    let text: String = lines[start..end]
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if text.is_empty() { None } else { Some(text) }
}

fn parse_experience(lines: &[&str]) -> Vec<Experience> {
    let (start, end) = match section_range(lines, &["EXPERIENCES", "EXPERIENCE", "WORK EXPERIENCE", "PROFESSIONAL EXPERIENCE"]) {
        Some(r) => r,
        None => return vec![],
    };

    // Pattern: "Title  Date - Date" on a single line
    // e.g. "Senior Engineer, Data Infrastructure May 2024 - Present"
    // e.g. "L3 Data Engineer Oct 2021 - Sep 2022"
    let date_range_re = Regex::new(
        r"(?i)((?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\.?\s+\d{4}|\d{4})\s*[-–—]\s*((?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\.?\s+\d{4}|\d{4}|Present|Current|Now)"
    ).unwrap();
    let bullet_re = Regex::new(r"^[•·▪\-]").unwrap();

    let mut experiences: Vec<Experience> = vec![];
    let section_lines = &lines[start..end];

    // State machine: we alternate between:
    //   state 0: expecting a company line
    //   state 1: expecting a title+date line (or another role at same company)
    //   state 2: collecting bullets
    let mut i = 0;
    while i < section_lines.len() {
        let line = section_lines[i].trim();
        i += 1;

        if line.is_empty() { continue; }

        // If this line contains a date range, it's a title line
        if date_range_re.is_match(line) {
            // Extract the date range
            let (start_date, end_date) = extract_dates(line, &date_range_re);
            // Everything before the date range = title
            let title = date_range_re.replace(line, "").trim().trim_end_matches(|c: char| !c.is_alphanumeric()).trim().to_string();

            // Company: the last non-bullet, non-date, non-empty line before this
            // It's already been pushed as a company when we processed the previous non-date line.
            // Strategy: look backwards in experiences to find the last one with empty company
            let company = if let Some(last) = experiences.last_mut() {
                if last.company.is_empty() && last.title.is_empty() {
                    // The last entry is a "company placeholder" — fill in the title
                    last.title = title.clone();
                    last.start_date = start_date;
                    last.end_date = end_date;
                    // Collect bullets
                    collect_bullets(section_lines, &mut i, &bullet_re, last);
                    continue;
                } else if last.company.is_empty() {
                    // Same company, new role
                    last.company.clone()
                } else {
                    last.company.clone()
                }
            } else {
                String::new()
            };

            let mut exp = Experience {
                title,
                company,
                location: None,
                start_date,
                end_date,
                bullets: vec![],
            };
            collect_bullets(section_lines, &mut i, &bullet_re, &mut exp);
            experiences.push(exp);
        } else if !bullet_re.is_match(line) {
            // This is a company/location line
            // Parse: "Company  Location" — split on multiple spaces or known location patterns
            let (company, location) = split_company_location(line);
            experiences.push(Experience {
                title: String::new(),
                company,
                location,
                start_date: None,
                end_date: None,
                bullets: vec![],
            });
        }
    }

    // Clean up: merge company-only placeholders with the next entry
    // Remove entries with no title (pure company lines that got their title inline)
    let mut result: Vec<Experience> = vec![];
    let mut pending_company: Option<(String, Option<String>)> = None;

    for exp in experiences {
        if exp.title.is_empty() {
            // This is a company placeholder
            pending_company = Some((exp.company.clone(), exp.location.clone()));
        } else {
            if let Some((co, loc)) = pending_company.take() {
                let mut e = exp;
                if e.company.is_empty() || e.company == co {
                    e.company = co;
                    if e.location.is_none() { e.location = loc; }
                }
                result.push(e);
            } else {
                result.push(exp);
            }
        }
    }

    result
}

fn collect_bullets(lines: &[&str], i: &mut usize, bullet_re: &Regex, exp: &mut Experience) {
    let date_re = Regex::new(r"(?i)(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\.?\s+\d{4}").unwrap();
    let date_range_re = Regex::new(r"(?i)(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\.?\s+\d{4}\s*[-–—]").unwrap();

    while *i < lines.len() {
        let line = lines[*i].trim();
        if line.is_empty() {
            *i += 1;
            continue;
        }
        if bullet_re.is_match(line) {
            let bullet = line.trim_start_matches(|c: char| c == '•' || c == '·' || c == '▪' || c == '-').trim().to_string();
            exp.bullets.push(bullet);
            *i += 1;
        } else {
            // Stop if: section header, date-range line (new job title), or
            // a short non-indented line that looks like a company/role header
            // (i.e., doesn't start with lowercase and contains no date)
            if date_range_re.is_match(line) || looks_like_section_header(line) {
                break;
            }
            // A short line (< 60 chars) that starts with a capital letter and has no
            // punctuation at the end is likely a company name — stop here
            let first_char = line.chars().next().unwrap_or(' ');
            let is_likely_company = first_char.is_uppercase()
                && line.len() < 70
                && !date_re.is_match(line)
                && !line.ends_with('.');
            if is_likely_company {
                break;
            }
            // Continuation bullet
            if let Some(last) = exp.bullets.last_mut() {
                last.push(' ');
                last.push_str(line);
            }
            *i += 1;
        }
    }
}

fn looks_like_section_header(line: &str) -> bool {
    let upper = line.trim().to_uppercase();
    ALL_SECTION_HEADERS.iter().any(|s| upper == *s)
}

fn extract_dates(line: &str, re: &Regex) -> (Option<String>, Option<String>) {
    if let Some(m) = re.find(line) {
        let range = m.as_str();
        // Split on the dash/em-dash
        let sep_re = Regex::new(r"\s*[-–—]\s*").unwrap();
        let parts: Vec<&str> = sep_re.splitn(range, 2).collect();
        if parts.len() == 2 {
            return (Some(parts[0].trim().to_string()), Some(parts[1].trim().to_string()));
        }
    }
    (None, None)
}

fn split_company_location(line: &str) -> (String, Option<String>) {
    // Common pattern: "Company  City, Country" or "Company City"
    // Try splitting on 2+ spaces
    let multi_space_re = Regex::new(r"\s{2,}").unwrap();
    let parts: Vec<&str> = multi_space_re.splitn(line, 2).collect();
    if parts.len() == 2 {
        return (parts[0].trim().to_string(), Some(parts[1].trim().to_string()));
    }

    // Try common location keywords at the end
    let location_re = Regex::new(r"\s+(Jakarta|Bandung|Surabaya|Indonesia|Singapore|Malaysia|Remote|[A-Z][a-z]+,\s+[A-Z][a-z]+)$").unwrap();
    if let Some(m) = location_re.find(line) {
        let company = line[..m.start()].trim().to_string();
        let location = m.as_str().trim().to_string();
        return (company, Some(location));
    }

    (line.to_string(), None)
}

fn parse_education(lines: &[&str]) -> Vec<Education> {
    let (start, end) = match section_range(lines, &["EDUCATION", "EDUCATIONAL BACKGROUND"]) {
        Some(r) => r,
        None => return vec![],
    };

    let date_range_re = Regex::new(
        r"(?i)((?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\.?\s+\d{4}|\d{4})\s*[-–—]\s*((?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\.?\s+\d{4}|\d{4}|Present)"
    ).unwrap();
    let gpa_re = Regex::new(r"(?i)GPA[:\s]+(\d+\.\d+)").unwrap();
    // "Bandung Institute of Technology, B.S. in Computer Science Aug 2017 - Oct 2021"
    let degree_re = Regex::new(r"(?i),?\s*(B\.?S\.?|M\.?S\.?|B\.?Sc\.?|M\.?Sc\.?|Ph\.?D\.?|S\.?T\.?|S\.?Kom\.?|Bachelor|Master|Diploma)\s+(?:in|of|–)?\s*([A-Za-z ]+?)(?:\s+(?:Aug|Sep|Oct|Nov|Dec|Jan|Feb|Mar|Apr|May|Jun|Jul)|\s*$)").unwrap();

    let mut educations: Vec<Education> = vec![];

    for line in &lines[start..end] {
        let t = line.trim();
        if t.is_empty() { continue; }

        // Skip bullet-style lines that are just notes (e.g., "Lab Assistant: ...")
        if t.starts_with("Lab") || t.starts_with("Teaching") || t.starts_with("Research") {
            if let Some(last) = educations.last_mut() {
                last.notes.push(t.to_string());
            }
            continue;
        }

        let clean = t.trim_start_matches(|c: char| c == '•' || c == '-' || c == '·').trim();

        // Extract dates
        let (start_date, end_date) = extract_dates(clean, &date_range_re);

        // Extract degree + field
        let (degree, field, institution) = if let Some(caps) = degree_re.captures(clean) {
            let degree = caps[1].trim().to_string();
            let field = caps[2].trim().to_string();
            // Institution = everything before the degree match
            let m = degree_re.find(clean).unwrap();
            let inst = clean[..m.start()].trim().trim_end_matches(',').trim().to_string();
            (Some(degree), Some(field), inst)
        } else {
            (None, None, clean.to_string())
        };

        let gpa = gpa_re.captures(clean).map(|c| c[1].to_string());

        educations.push(Education {
            institution,
            degree,
            field,
            start_date,
            end_date,
            gpa,
            notes: vec![],
        });
    }

    educations
}

fn parse_skills(lines: &[&str]) -> Skills {
    let (start, end) = match section_range(lines, &["SKILLS", "TECHNICAL SKILLS", "TECHNOLOGIES", "TECH STACK"]) {
        Some(r) => r,
        None => return Skills { languages: vec![], frameworks: vec![], databases: vec![], tools: vec![], cloud: vec![], other: vec![] },
    };

    let mut skills = Skills {
        languages: vec![],
        frameworks: vec![],
        databases: vec![],
        tools: vec![],
        cloud: vec![],
        other: vec![],
    };

    for line in &lines[start..end] {
        let t = line.trim();
        if t.is_empty() { continue; }

        if let Some(colon_pos) = t.find(':') {
            let category = t[..colon_pos].trim().to_uppercase();
            let items: Vec<String> = t[colon_pos + 1..]
                .split(|c| c == ',' || c == '|' || c == '/')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if category.contains("LANGUAGE") || category.contains("PROGRAMMING") {
                skills.languages.extend(items);
            } else if category.contains("FRAMEWORK") || category.contains("LIBRARY") {
                skills.frameworks.extend(items);
            } else if category.contains("DATABASE") || category.contains("DB") {
                skills.databases.extend(items);
            } else if category.contains("CLOUD") || category.contains("AWS") || category.contains("GCP") {
                skills.cloud.extend(items);
            } else if category.contains("TOOL") || category.contains("DEVOPS") {
                skills.tools.extend(items);
            } else {
                skills.other.extend(items);
            }
        } else {
            let items: Vec<String> = t
                .split(|c| c == ',' || c == '|')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            skills.other.extend(items);
        }
    }

    skills
}

fn parse_projects(lines: &[&str]) -> Vec<Project> {
    let (start, end) = match section_range(lines, &["PROJECTS", "PERSONAL PROJECTS", "SIDE PROJECTS"]) {
        Some(r) => r,
        None => return vec![],
    };

    // Pattern: "• Project Name  Description"
    let tech_re = Regex::new(
        r"(?i)\b(Python|Rust|Go|Java|Scala|Spark|Kafka|Airflow|dbt|Postgres|MySQL|Redis|Docker|Kubernetes|AWS|GCP|Azure|Terraform|Flink|BigQuery|Snowflake|Redshift|Hive|Hadoop|Databricks|FastAPI|Django|React|TypeScript|JavaScript|Bash)\b"
    ).unwrap();

    let mut projects: Vec<Project> = vec![];

    for line in &lines[start..end] {
        let t = line.trim();
        if t.is_empty() { continue; }
        let clean = t.trim_start_matches(|c: char| c == '•' || c == '-' || c == '·').trim();
        if clean.is_empty() { continue; }

        // Try splitting "Name  Description":
        // 1. First try 2+ spaces
        // 2. Then try splitting on a capitalized description-starter word following the name
        let multi_space_re = Regex::new(r"\s{2,}").unwrap();
        let desc_split_re = Regex::new(r"\s+(?:A |An |The |Developed|Built|Created|Designed|Implemented|A\s|An\s|Digital|Open|Personal|Simple|Tool|Web|API|CLI|Full)").unwrap();
        let (name, description) = if let Some(m) = multi_space_re.find(clean) {
            (clean[..m.start()].trim().to_string(), Some(clean[m.end()..].trim().to_string()))
        } else if let Some(m) = desc_split_re.find(clean) {
            (clean[..m.start()].trim().to_string(), Some(clean[m.start()..].trim().to_string()))
        } else {
            (clean.to_string(), None)
        };

        let technologies: Vec<String> = tech_re
            .find_iter(clean)
            .map(|m| m.as_str().to_string())
            .collect();

        projects.push(Project { name, description, technologies });
    }

    projects
}

fn parse_achievements(lines: &[&str]) -> Vec<String> {
    let (start, end) = match section_range(lines, &["ACHIEVEMENTS", "CERTIFICATIONS", "CERTIFICATES", "AWARDS"]) {
        Some(r) => r,
        None => return vec![],
    };

    lines[start..end]
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| l.trim_start_matches(|c: char| c == '•' || c == '-' || c == '·' || c == '▪').trim().to_string())
        .collect()
}
