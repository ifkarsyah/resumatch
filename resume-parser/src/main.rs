mod model;
mod parser;

use anyhow::Result;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let pdf_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("Ferdian_Ifkarsyah___Resume___2025___11____DE.pdf")
    };

    let text = pdf_extract::extract_text(&pdf_path)?;
    let resume = parser::parse_resume(&text);
    let json = serde_json::to_string_pretty(&resume)?;
    println!("{}", json);

    Ok(())
}
