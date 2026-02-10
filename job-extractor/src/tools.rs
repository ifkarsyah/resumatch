const KNOWN_TOOLS: &[&str] = &[
    "Kafka", "Spark", "Flink", "Python", "Scala", "Java", "Go", "Rust",
    "SQL", "PostgreSQL", "MySQL", "MongoDB", "Redis", "Elasticsearch",
    "Snowflake", "dbt", "Airflow", "Kubernetes", "Docker", "Terraform",
    "AWS", "GCP", "Azure", "Hadoop", "Hive", "Trino", "Presto",
    "Databricks", "Redshift", "BigQuery", "Kinesis", "Pulsar",
];

pub fn extract_tools(text: &str) -> Vec<String> {
    KNOWN_TOOLS
        .iter()
        .filter(|&&tool| is_word_match(text, tool))
        .map(|&t| t.to_string())
        .collect()
}

/// Case-insensitive whole-word search.
fn is_word_match(text: &str, word: &str) -> bool {
    let lower_text = text.to_lowercase();
    let lower_word = word.to_lowercase();
    let mut start = 0;

    while let Some(pos) = lower_text[start..].find(&lower_word) {
        let abs = start + pos;
        let before_ok = abs == 0
            || !lower_text[..abs]
                .chars()
                .last()
                .map(|c| c.is_alphanumeric())
                .unwrap_or(false);
        let after_ok = abs + lower_word.len() >= lower_text.len()
            || !lower_text[abs + lower_word.len()..]
                .chars()
                .next()
                .map(|c| c.is_alphanumeric())
                .unwrap_or(false);
        if before_ok && after_ok {
            return true;
        }
        start = abs + 1;
    }
    false
}
