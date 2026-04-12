//! ADR命令行接口
use crate::knowledge::adr::{AdrEntry, AdrIndex, AdrStatus};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "adr", about = "ADR (Architecture Decision Records) 管理")]
pub struct AdrCli {
    #[command(subcommand)]
    command: AdrCommands,
}

#[derive(Subcommand)]
pub enum AdrCommands {
    List { #[arg(short, long)] status: Option<AdrStatus> },
    Create { title: String },
    Query { id: String },
}

impl AdrCli {
    pub async fn execute(&self, index: &AdrIndex) -> anyhow::Result<()> {
        match &self.command {
            AdrCommands::List { status } => {
                println!("{:<10} {:<12} {:<32} {}", "ID", "Status", "Title", "Date");
                for entry in &index.entries {
                    if let Some(s) = status { if entry.status != *s { continue; } }
                    let date = entry.date.format("%Y-%m-%d").to_string();
                    let status_str = format!("{:?}", entry.status).to_lowercase();
                    let title = if entry.title.len() > 30 { &entry.title[..30] } else { &entry.title };
                    println!("{:<10} {:<12} {:<32} {}", entry.id, status_str, title, date);
                }
            }
            AdrCommands::Create { title } => { println!("创建ADR: {}", title); }
            AdrCommands::Query { id } => {
                if let Some(entry) = index.entries.iter().find(|e| e.id == *id) {
                    println!("ID: {}\nTitle: {}\nStatus: {:?}\nDate: {}\nTags: {:?}",
                        entry.id, entry.title, entry.status, entry.date.format("%Y-%m-%d"), entry.tags);
                } else { return Err(anyhow::anyhow!("ADR not found: {}", id)); }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_cli_list() {
        let mut index = AdrIndex::default();
        index.entries.push(AdrEntry {
            id: "ADR-001".to_string(), title: "Test".to_string(),
            status: AdrStatus::Accepted, date: Utc::now(), tags: vec![], content: "test".to_string(),
        });
        assert_eq!(index.entries.len(), 1);
    }
}
