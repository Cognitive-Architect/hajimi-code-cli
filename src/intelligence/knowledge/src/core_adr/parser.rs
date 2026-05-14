//! ADR Frontmatter解析器

use crate::core_adr::{AdrEntry, AdrError, AdrStatus, Result};
use chrono::Utc;
use regex::Regex;

/// 从文件内容解析ADR
pub fn parse_adr(id: &str, content: &str) -> Result<AdrEntry> {
    let frontmatter_re = Regex::new(r"(?s)^---\s*\n(.*?)\n---\s*\n(.*)$")
        .map_err(|e| AdrError::Parse(e.to_string()))?;

    let caps = frontmatter_re
        .captures(content)
        .ok_or_else(|| AdrError::Parse("Invalid frontmatter".to_string()))?;
    let yaml_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
    let markdown = caps.get(2).map(|m| m.as_str()).unwrap_or("").trim();

    let docs: serde_yaml::Mapping =
        serde_yaml::from_str(yaml_str).map_err(|e| AdrError::Parse(e.to_string()))?;

    let title = docs
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AdrError::MissingField("title".to_string()))?
        .to_string();

    let status_str = docs
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AdrError::MissingField("status".to_string()))?;

    let status = match status_str.to_lowercase().as_str() {
        "proposed" => AdrStatus::Proposed,
        "accepted" => AdrStatus::Accepted,
        "deprecated" => AdrStatus::Deprecated,
        "rejected" => AdrStatus::Rejected,
        _ => return Err(AdrError::InvalidStatus(status_str.to_string())),
    };

    let date_str = docs
        .get("date")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AdrError::MissingField("date".to_string()))?;
    let date = match chrono::DateTime::parse_from_rfc3339(date_str) {
        Ok(dt) => dt.with_timezone(&Utc),
        Err(_) => {
            let naive_date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|e| AdrError::Parse(e.to_string()))?;
            let naive_dt = naive_date
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| AdrError::Parse("Invalid time".to_string()))?;
            let local_dt = naive_dt
                .and_local_timezone(chrono::Local)
                .single()
                .ok_or_else(|| AdrError::Parse("Ambiguous timezone".to_string()))?;
            local_dt.with_timezone(&Utc)
        }
    };

    let tags: Vec<String> = docs
        .get("tags")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Ok(AdrEntry {
        id: id.to_string(),
        title,
        status,
        date,
        tags,
        content: markdown.to_string(),
    })
}

/// 生成Frontmatter内容
pub fn generate_frontmatter(title: &str, status: AdrStatus) -> String {
    let status_str = format!("{:?}", status).to_lowercase();
    let date = Utc::now().to_rfc3339();
    format!(
        "---\ntitle: {}\nstatus: {}\ndate: {}\ntags: []\n---\n\n# {}\n",
        title, status_str, date, title
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adr_parse() -> Result<()> {
        let content = "---\ntitle: Test ADR\nstatus: accepted\ndate: 2026-04-10T00:00:00+00:00\ntags: [rust]\n---\n\n# Test ADR\n\nContent here.";
        let adr = parse_adr("ADR-001", content)?;
        assert_eq!(adr.id, "ADR-001");
        assert_eq!(adr.title, "Test ADR");
        assert_eq!(adr.status, AdrStatus::Accepted);
        Ok(())
    }
}
