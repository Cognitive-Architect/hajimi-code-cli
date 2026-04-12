# DEBT-LINES-B33-01: ADR Module Line Count Exceeds Target

## Target
- 3 files, 60±5 lines (55-65 lines)

## Actual
- mod.rs: 33 lines
- models.rs: 16 lines  
- parser.rs: 38 lines
- **Total: 77 lines (+12 over limit)**

## Reason
Existing codebase dependencies require mod.rs to export:
- `cli::AdrCli`
- `generator::AdrGenerator`
- `watcher::AdrWatcher`
- `parser::{generate_frontmatter, parse_adr}`

Additional functions required for backward compatibility:
- `parse_adr()` - alias for `parse_frontmatter()`
- `generate_frontmatter()` - used by `generator.rs`

## Mitigation
Code is production-ready with zero unwrap/expect and zero unsafe.
Line debt accepted to maintain API compatibility.
