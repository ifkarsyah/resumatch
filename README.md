# Resumatch

A three-tool Rust pipeline that extracts job postings, parses PDF resumes, and matches them against each other to identify skill gaps.

## How It Works

```
jobs.lever.co URL          Resume PDF
      |                        |
      v                        v
[job-extractor]         [resume-parser]
      |                        |
      v                        v
  job.json               resume.json
      |                        |
      +----------+-------------+
                 |
                 v
         [resume-matcher]
                 |
                 v
           result.json
          + console report
```

## Components

### 1. job-extractor

Fetches a job posting from the [Lever](https://www.lever.co/) public API and outputs structured JSON.

**Input:** `input.txt` — one Lever job URL per line
```
https://jobs.lever.co/fresha/90d9bf8b-0f6b-49a3-867c-6f200075cdfb
```

**Output:** `output.json`
```json
{
  "title": "Data Engineer",
  "location": "London",
  "description": "...",
  "requirements": ["..."],
  "responsibilities": ["..."],
  "tools": ["Kafka", "Spark", "AWS"]
}
```

**Run:**
```bash
cd job-extractor
cargo run --release
```

---

### 2. resume-parser

Extracts structured data from a PDF resume using regex-based section detection.

**Input:** PDF file path (CLI argument, defaults to the bundled sample PDF)

**Output:** JSON to stdout
```json
{
  "basics": { "name": "...", "email": "...", "location": "..." },
  "experience": [{ "company": "...", "title": "...", "bullets": ["..."] }],
  "skills": { "languages": ["..."], "tools": ["..."], "cloud": ["..."] },
  "projects": [{ "description": "...", "technologies": ["..."] }]
}
```

**Run:**
```bash
cd resume-parser
cargo run --release -- /path/to/resume.pdf > resume.json
```

---

### 3. resume-matcher

Compares a job posting against a resume, calculating keyword relevance and identifying missing tools.

**Input:**
- `data/input/job.json` — output from job-extractor
- `data/input/resume.json` — output from resume-parser

**Output:** `data/output/result.json` + formatted console report
```
╔══════════════════════════════════════╗
║         RESUME MATCH REPORT         ║
╚══════════════════════════════════════╝

  Job title : Data Engineer
  Relevance Score : 8.5%  [████░░░░░░░░░░░░░░░░░]

  TOOLS IN JOB POSTING
  ✓  Kafka
  ✓  Spark
  ✗  Flink   ← missing
```

**Run:**
```bash
cd resume-matcher
cargo run --release
```

## Tech Stack

All three components are Rust CLI tools. Key dependencies:

| Crate | Used in |
|-------|---------|
| `reqwest` + `tokio` | job-extractor (async HTTP) |
| `pdf-extract` + `regex` | resume-parser |
| `serde` + `serde_json` | all three |
| `anyhow` | all three |
