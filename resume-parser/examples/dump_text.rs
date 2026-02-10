fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).map(|s| s.as_str()).unwrap_or("Ferdian_Ifkarsyah___Resume___2025___11____DE.pdf");
    let text = pdf_extract::extract_text(path)?;
    for (i, line) in text.lines().enumerate() {
        println!("{:4}: {:?}", i+1, line);
    }
    Ok(())
}
