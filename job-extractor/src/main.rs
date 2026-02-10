mod extractor;
mod lever;
mod models;
mod tools;

use std::path::Path;

use extractor::extract_job;

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
