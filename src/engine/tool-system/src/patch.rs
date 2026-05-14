//! Patch apply module - B-W10/03: UDF & HDiff atomic apply
use super::{ToolError, ToolErrorKind};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum PatchFormat {
    Unified,
    HDiff,
}
#[derive(Debug, Clone)]
pub struct ConflictMarker {
    pub file: String,
    pub line: usize,
    pub context: String,
}

pub struct PatchResult {
    pub conflicts: Vec<ConflictMarker>,
    pub new_content: Option<String>,
}

pub fn apply_patch(
    file_path: &Path,
    patch: &str,
    format: PatchFormat,
) -> Result<PatchResult, ToolError> {
    let content = std::fs::read_to_string(file_path).map_err(|e| ToolError {
        message: format!("Read: {}", e),
        kind: ToolErrorKind::ExecutionFailed,
    })?;
    let lines: Vec<&str> = content.lines().collect();
    let mut result = match format {
        PatchFormat::Unified => apply_unified(&lines, patch)?,
        PatchFormat::HDiff => apply_hdiff(&lines, patch)?,
    };
    if result.conflicts.is_empty() {
        let new_content = result.new_content.take().unwrap_or_default();
        let temp = file_path.with_extension("tmp");
        std::fs::write(&temp, &new_content).map_err(|e| ToolError {
            message: format!("Write: {}", e),
            kind: ToolErrorKind::ExecutionFailed,
        })?;
        std::fs::rename(&temp, file_path).map_err(|e| ToolError {
            message: format!("Rename: {}", e),
            kind: ToolErrorKind::ExecutionFailed,
        })?;
    }
    Ok(result)
}

fn apply_unified(lines: &[&str], patch: &str) -> Result<PatchResult, ToolError> {
    let mut result = Vec::new();
    let mut conflicts = Vec::new();
    let mut i = 0usize;
    let mut in_hunk = false;
    let mut ctx_lines: Vec<&str> = Vec::new();
    let mut old_start = 0usize;
    for line in patch.lines() {
        if line.starts_with("@@") {
            in_hunk = true;
            ctx_lines.clear();
            if let Some(m) = line.split_whitespace().nth(1) {
                let nums: Vec<&str> = m[1..].split(',').collect();
                old_start = nums[0].parse::<usize>().unwrap_or(1).saturating_sub(1);
            }
            i = fuzzy_match(lines, &[], old_start);
        } else if in_hunk {
            match line.chars().next() {
                Some(' ') => {
                    if i < lines.len() {
                        if lines[i] == &line[1..] {
                            result.push(&line[1..]);
                            i += 1;
                        } else {
                            conflicts.push(ConflictMarker {
                                file: "".into(),
                                line: i + 1,
                                context: line.into(),
                            });
                        }
                    }
                }
                Some('-') => {
                    if i < lines.len() && lines[i] == &line[1..] {
                        i += 1;
                    } else {
                        conflicts.push(ConflictMarker {
                            file: "".into(),
                            line: i + 1,
                            context: line.into(),
                        });
                    }
                }
                Some('+') => result.push(&line[1..]),
                Some('\n') | Some('\r') => {}
                _ => {
                    if line.is_empty() {
                        result.push("");
                    }
                }
            }
        }
    }
    if !conflicts.is_empty() {
        return Ok(PatchResult {
            conflicts,
            new_content: None,
        });
    }
    while i < lines.len() {
        result.push(lines[i]);
        i += 1;
    }
    Ok(PatchResult {
        conflicts: Vec::new(),
        new_content: Some(result.join("\n")),
    })
}

fn apply_hdiff(lines: &[&str], patch: &str) -> Result<PatchResult, ToolError> {
    let mut result: Vec<&str> = lines.to_vec();
    let mut conflicts = Vec::new();
    let mut seen_hashes: HashSet<u64> = HashSet::new();
    for line in patch.lines() {
        if line.starts_with("#") || line.is_empty() {
            continue;
        }
        if let Some(stripped) = line.strip_prefix("HASH:") {
            let h: u64 = stripped.parse().map_err(|_| ToolError {
                message: "Bad hash".into(),
                kind: ToolErrorKind::InvalidPatchFormat,
            })?;
            if !seen_hashes.insert(h) {
                return Err(ToolError {
                    message: "Circular patch".into(),
                    kind: ToolErrorKind::InvalidPatchFormat,
                });
            }
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() < 2 {
            continue;
        }
        let line_no: usize = parts[0].parse().map_err(|_| ToolError {
            message: "Bad line".into(),
            kind: ToolErrorKind::InvalidPatchFormat,
        })?;
        let action = parts[1];
        let content = parts.get(2).copied().unwrap_or("");
        match action {
            "DEL" => {
                if line_no > 0 && line_no <= result.len() && result[line_no - 1] == content {
                    result.remove(line_no - 1);
                } else {
                    let ctx = if line_no > 0 && line_no <= result.len() {
                        result[line_no - 1]
                    } else {
                        ""
                    };
                    conflicts.push(ConflictMarker {
                        file: "".into(),
                        line: line_no,
                        context: ctx.into(),
                    });
                }
            }
            "INS" => {
                if line_no <= result.len() {
                    result.insert(line_no - 1, content);
                } else {
                    result.push(content);
                }
            }
            "REP" => {
                if line_no > 0 && line_no <= result.len() {
                    result[line_no - 1] = content;
                } else {
                    conflicts.push(ConflictMarker {
                        file: "".into(),
                        line: line_no,
                        context: "REP".into(),
                    });
                }
            }
            _ => {}
        }
    }
    if !conflicts.is_empty() {
        return Ok(PatchResult {
            conflicts,
            new_content: None,
        });
    }
    Ok(PatchResult {
        conflicts: Vec::new(),
        new_content: Some(result.join("\n")),
    })
}

pub fn fuzzy_match(lines: &[&str], context: &[&str], start: usize) -> usize {
    if context.is_empty() {
        return start;
    }
    let ctx_len = context.len();
    let search_start = start.saturating_sub(3);
    let search_end = (start + 3).min(lines.len().saturating_sub(ctx_len) + 1);
    for offset in search_start..search_end {
        if lines[offset..offset + ctx_len] == *context {
            return offset;
        }
    }
    start
}

pub fn generate_conflict_markers(conflicts: &[ConflictMarker], original: &str) -> String {
    let mut out = String::new();
    let lines: Vec<&str> = original.lines().collect();
    for c in conflicts {
        out.push_str(&format!("<<<<<<< HEAD (line {})\n", c.line));
        if c.line > 0 && c.line <= lines.len() {
            out.push_str(lines[c.line - 1]);
            out.push('\n');
        }
        out.push_str("=======\n");
        out.push_str(&c.context);
        out.push('\n');
        out.push_str(">>>>>>> PATCH\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fuzzy_match_exact() {
        assert_eq!(fuzzy_match(&["a", "b", "c"], &["b"], 1), 1);
    }
    #[test]
    fn test_fuzzy_match_offset() {
        assert_eq!(fuzzy_match(&["a", "b", "c", "d"], &["c"], 2), 2);
    }
}
