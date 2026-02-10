# resume-matcher

A Rust CLI that compares a job posting against a resume and reports:

- **Missing tools** — tools mentioned in the job but absent from the resume
- **Found tools** — tools mentioned in the job that appear on the resume
- **Relevance score** — keyword overlap between the job posting and resume text (0–100%)
- **Matched keywords** — the specific job keywords found in the resume

## Directory layout

```
resume-matcher/
├── data/
│   ├── input/
│   │   ├── job.json       ← job posting (input)
│   │   └── resume.json    ← your resume (input)
│   └── output/
│       └── result.json    ← analysis result (generated)
└── src/
    └── main.rs
```

## Input formats

### `data/input/job.json`

```json
{
  "title": "Data Engineer",
  "description": "...",
  "requirements": ["5+ years experience", "..."],
  "responsibilities": ["Design pipelines", "..."],
  "tools": ["Kafka", "Spark", "Terraform"],
  ...
}
```

The `tools` array drives the found/missing breakdown. `description`, `requirements`, and `responsibilities` are all scanned for keyword matching.

### `data/input/resume.json`

```json
{
  "experience": [
    {
      "bullets": ["Built Kafka pipelines ...", "..."]
    }
  ],
  "skills": {
    "languages": [],
    "frameworks": [],
    "databases": [],
    "tools": [],
    "cloud": [],
    "other": []
  },
  "projects": [
    {
      "description": "...",
      "technologies": []
    }
  ]
}
```

All bullet text, skill entries, and project technologies are scanned.

## Output — `data/output/result.json`

```json
{
  "missing_tools": ["Flink", "Azure"],
  "found_tools": ["Kafka", "Spark", "Terraform", "AWS", "GCP"],
  "relevance_score": 8.53,
  "matched_keywords": ["data", "infrastructure", "kafka", "..."],
  "total_job_keywords": 211
}
```

| Field | Description |
|---|---|
| `missing_tools` | Tools from `job.tools[]` not found anywhere in resume text |
| `found_tools` | Tools from `job.tools[]` found in resume text |
| `relevance_score` | `matched / total_job_keywords × 100` |
| `matched_keywords` | Job keywords (>3 chars, no stop-words) present in resume |
| `total_job_keywords` | Total unique keywords extracted from the job posting |

## Usage

```bash
# build
cargo build --release

# run (reads data/input/, writes data/output/result.json)
./target/release/resume-matcher

# or via cargo
cargo run --release
```

The tool always reads from `data/input/job.json` and `data/input/resume.json`,
and writes output to `data/output/result.json` (directory is created automatically).

## Tips for a better score

- Fill in the `skills.*` arrays in `resume.json` — many tools you use are buried in bullets and won't surface as tokens unless they appear verbatim.
- Match the exact capitalisation used in the job posting (e.g. `Kafka` vs `kafka` — the tool normalises to lowercase for comparison, so either works).
- Add keywords from the job's `responsibilities` and `requirements` sections to your bullet points where relevant.
