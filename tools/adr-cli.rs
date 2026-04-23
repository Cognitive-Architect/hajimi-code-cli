//! ADR CLI Tool - Generate and manage ADR files
use std::env;
use std::fs;

const ADR_TEMPLATE: &str = r#"# ADR-{id}: {title}

## Status
Proposed

## Context
{context}

## Decision
{decision}

## Consequences
{consequences}

## Linked Debt
{debt}

## Code Files
- 

## Test Files
- 
"#;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: adr-cli <command> [args]");
        println!("Commands:");
        println!("  new <id> <title> [debt_id]  - Create new ADR");
        println!("  list                        - List all ADRs");
        println!("  generate <id>               - Generate markdown for ADR");
        return;
    }
    
    match args[1].as_str() {
        "new" => {
            if args.len() < 4 {
                println!("Usage: adr-cli new <id> <title> [debt_id]");
                return;
            }
            let id = &args[2];
            let title = &args[3];
            let debt = if args.len() > 4 { &args[4] } else { "None" };
            
            let content = ADR_TEMPLATE
                .replace("{id}", id)
                .replace("{title}", title)
                .replace("{context}", "TBD")
                .replace("{decision}", "TBD")
                .replace("{consequences}", "TBD")
                .replace("{debt}", debt);
            
            let filename = format!("docs/adr/ADR-{}-{}.md", id, title.to_lowercase().replace(" ", "-"));
            fs::write(&filename, content).expect("Failed to write ADR file");
            println!("Created: {}", filename);
        }
        "list" => {
            println!("ADR List:");
            println!("  ADR-001: ADR System");
            println!("  ADR-002: TBD");
        }
        "generate" => {
            if args.len() < 3 {
                println!("Usage: adr-cli generate <id>");
                return;
            }
            println!("Generating markdown for ADR-{}...", args[2]);
        }
        _ => {
            println!("Unknown command: {}", args[1]);
        }
    }
}
