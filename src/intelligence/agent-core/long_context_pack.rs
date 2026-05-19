//! LongContextPack — Structure, build rules, default exclude lists, and truncation policies
//! for multi-megabyte long context assemblies.
//!
//! Designed for the Hajimi IDE v1 1M long context workflow. Pure synchronous logic,
//! zero dependency on interface or network layers.

use crate::context_window_manager::{estimate_tokens, ContentType, ContextBlock, ContextPriority};
use std::fs;
use std::path::{Path, PathBuf};

/// Sources of context blocks within the pack to allow visual audit and traceability.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ContextSource {
    /// A structured directory tree representation of the codebase.
    RepoTree,
    /// An active, currently open workspace file.
    ActiveFile(String),
    /// A user-explicitly selected file or folder.
    UserProvided(String),
    /// Content retrieved directly from the blackboard state.
    Blackboard,
    /// Custom external source with descriptive name.
    Other(String),
}

impl ContextSource {
    /// Encodes source information into the block name for visual traceability (e.g. source:sub-info:filename)
    pub fn encode_name(&self, block_name: &str) -> String {
        match self {
            Self::RepoTree => format!("repo_tree:repo_tree:{}", block_name),
            Self::ActiveFile(p) => format!("active_file:{}:{}", p, block_name),
            Self::UserProvided(p) => format!("user_provided:{}:{}", p, block_name),
            Self::Blackboard => format!("blackboard:blackboard:{}", block_name),
            Self::Other(d) => format!("other:{}:{}", d, block_name),
        }
    }
}

/// Policy describing how to handle files that exceed the defined max size boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LargeFilePolicy {
    /// Skip the file entirely, registering an omitted block with an explicit reason.
    Skip,
    /// Extract the head (first N lines) and tail (last N lines) of the file with divider info.
    HeadTail { lines: usize },
    /// Read the file fully into the pack regardless of size.
    Full,
}

/// Metadata-rich block wrapped inside the LongContextPack before final bridge assembly.
#[derive(Debug, Clone)]
pub struct PackedBlock {
    /// The standard ContextBlock that can be fed into the ContextWindowManager.
    pub block: ContextBlock,
    /// Origin category of this block.
    pub source: ContextSource,
    /// Original file system path if applicable.
    pub original_path: Option<PathBuf>,
    /// Whether this file was omitted or partially skipped during building.
    pub omitted: bool,
    /// Human-readable reason for omission or truncation (e.g., "File too large: 2.1MB").
    pub omitted_reason: Option<String>,
}

/// A structured container holding multiple high-priority and repository-wide context blocks.
#[derive(Debug, Clone)]
pub struct LongContextPack {
    /// Collection of packed blocks ready for priority-based layout or bridge ingestion.
    pub blocks: Vec<PackedBlock>,
    /// Aggregated heuristic token estimate for the entire package.
    pub total_token_estimate: usize,
}

impl LongContextPack {
    /// Empty pack initializer.
    pub fn empty() -> Self {
        Self {
            blocks: Vec::new(),
            total_token_estimate: 0,
        }
    }

    /// Converts this pack into flat context blocks compatible with `ContextWindowManager`.
    pub fn to_context_blocks(&self) -> Vec<ContextBlock> {
        self.blocks
            .iter()
            .map(|packed| {
                let mut b = packed.block.clone();
                b.name = packed.source.encode_name(&b.name);
                // If omitted, ensure content is empty or displays the skipped reason
                if packed.omitted {
                    b.content = format!(
                        "[OMITTED block: {} | Reason: {}]",
                        packed.block.name,
                        packed.omitted_reason.as_deref().unwrap_or("unknown")
                    );
                    b.token_estimate = estimate_tokens(&b.content);
                }
                b
            })
            .collect()
    }
}

/// Builder for constructing long context packages, implementing default excludes
/// and large file strategies.
#[derive(Debug, Clone)]
pub struct LongContextPackBuilder {
    blocks: Vec<PackedBlock>,
    exclude_patterns: Vec<String>,
    exclude_extensions: Vec<String>,
    max_file_size_bytes: u64,
    large_file_policy: LargeFilePolicy,
}

impl Default for LongContextPackBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LongContextPackBuilder {
    /// Create a new builder with pre-configured default excludes and policies.
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            // P0 redline excludes: don't load build artifacts, VCS, or packages
            exclude_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".cache".to_string(),
                "coverage".to_string(),
            ],
            // Exclude binaries to avoid polluting tokens
            exclude_extensions: vec![
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "mp4".to_string(),
                "zip".to_string(),
                "tar".to_string(),
                "gz".to_string(),
                "exe".to_string(),
                "dll".to_string(),
                "so".to_string(),
                "dylib".to_string(),
                "bin".to_string(),
                "pdf".to_string(),
                "ico".to_string(),
            ],
            max_file_size_bytes: 80_000, // 80KB max baseline
            large_file_policy: LargeFilePolicy::HeadTail { lines: 60 },
        }
    }

    /// Add custom exclude substring pattern.
    pub fn exclude_pattern(mut self, pattern: &str) -> Self {
        self.exclude_patterns.push(pattern.to_string());
        self
    }

    /// Add custom exclude file extension.
    pub fn exclude_extension(mut self, ext: &str) -> Self {
        self.exclude_extensions.push(ext.to_lowercase());
        self
    }

    /// Set max file size threshold before large file policies trigger.
    pub fn max_file_size(mut self, bytes: u64) -> Self {
        self.max_file_size_bytes = bytes;
        self
    }

    /// Set large file handling strategy.
    pub fn large_file_policy(mut self, policy: LargeFilePolicy) -> Self {
        self.large_file_policy = policy;
        self
    }

    /// Checks if a file path matches any exclusion criteria.
    pub fn should_exclude(&self, path: &Path) -> bool {
        // 1. Extension check
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let lower_ext = ext.to_lowercase();
            if self.exclude_extensions.iter().any(|e| e == &lower_ext) {
                return true;
            }
        }

        // 2. Component exact check
        for component in path.components() {
            if let Some(comp_str) = component.as_os_str().to_str() {
                if self.exclude_patterns.iter().any(|p| p == comp_str) {
                    return true;
                }
            }
        }
        false
    }

    /// Add a single file to the pack with rich validation and fallback strategies.
    pub fn add_file(
        &mut self,
        path: &Path,
        source: ContextSource,
        priority: ContextPriority,
    ) -> std::io::Result<()> {
        if self.should_exclude(path) {
            return Ok(());
        }

        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                let reason = format!("Metadata error: {}", e);
                self.blocks.push(PackedBlock {
                    block: ContextBlock {
                        name: path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        priority,
                        content_type: ContentType::Text,
                        content: String::new(),
                        token_estimate: 0,
                        truncatable: true,
                    },
                    source,
                    original_path: Some(path.to_path_buf()),
                    omitted: true,
                    omitted_reason: Some(reason),
                });
                return Ok(());
            }
        };

        if !metadata.is_file() {
            return Ok(());
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown_file")
            .to_string();

        let file_size = metadata.len();

        if file_size > self.max_file_size_bytes {
            match self.large_file_policy {
                LargeFilePolicy::Skip => {
                    let reason = format!(
                        "File size {} exceeds limit {}",
                        file_size, self.max_file_size_bytes
                    );
                    let block = ContextBlock {
                        name: file_name.clone(),
                        priority,
                        content_type: ContentType::Text,
                        content: String::new(),
                        token_estimate: 0,
                        truncatable: true,
                    };
                    self.blocks.push(PackedBlock {
                        block,
                        source,
                        original_path: Some(path.to_path_buf()),
                        omitted: true,
                        omitted_reason: Some(reason),
                    });
                }
                LargeFilePolicy::HeadTail { lines } => {
                    let content = match fs::read_to_string(path) {
                        Ok(c) => c,
                        Err(e) => {
                            let reason = format!("Read error: {}", e);
                            self.blocks.push(PackedBlock {
                                block: ContextBlock {
                                    name: file_name,
                                    priority,
                                    content_type: ContentType::Text,
                                    content: String::new(),
                                    token_estimate: 0,
                                    truncatable: true,
                                },
                                source,
                                original_path: Some(path.to_path_buf()),
                                omitted: true,
                                omitted_reason: Some(reason),
                            });
                            return Ok(());
                        }
                    };
                    let all_lines: Vec<&str> = content.lines().collect();
                    let total_lines = all_lines.len();

                    let processed_content = if total_lines <= lines * 2 {
                        content
                    } else {
                        let head = &all_lines[..lines];
                        let tail = &all_lines[total_lines - lines..];
                        let mut merged = head.join("\n");
                        merged.push_str(&format!(
                            "\n\n... [TRUNCATED {} lines due to file size {} bytes] ...\n\n",
                            total_lines - lines * 2,
                            file_size
                        ));
                        merged.push_str(&tail.join("\n"));
                        merged
                    };

                    let token_est = estimate_tokens(&processed_content);
                    let block = ContextBlock {
                        name: file_name.clone(),
                        priority,
                        content_type: ContentType::Text,
                        content: processed_content,
                        token_estimate: token_est,
                        truncatable: true,
                    };
                    self.blocks.push(PackedBlock {
                        block,
                        source,
                        original_path: Some(path.to_path_buf()),
                        omitted: false,
                        omitted_reason: Some(format!("Head-tail truncated to {} lines", lines * 2)),
                    });
                }
                LargeFilePolicy::Full => {
                    let content = match fs::read_to_string(path) {
                        Ok(c) => c,
                        Err(e) => {
                            let reason = format!("Read error: {}", e);
                            self.blocks.push(PackedBlock {
                                block: ContextBlock {
                                    name: file_name,
                                    priority,
                                    content_type: ContentType::Text,
                                    content: String::new(),
                                    token_estimate: 0,
                                    truncatable: true,
                                },
                                source,
                                original_path: Some(path.to_path_buf()),
                                omitted: true,
                                omitted_reason: Some(reason),
                            });
                            return Ok(());
                        }
                    };
                    let token_est = estimate_tokens(&content);
                    let block = ContextBlock {
                        name: file_name,
                        priority,
                        content_type: ContentType::Text,
                        content,
                        token_estimate: token_est,
                        truncatable: true,
                    };
                    self.blocks.push(PackedBlock {
                        block,
                        source,
                        original_path: Some(path.to_path_buf()),
                        omitted: false,
                        omitted_reason: None,
                    });
                }
            }
        } else {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    let reason = format!("Read error: {}", e);
                    self.blocks.push(PackedBlock {
                        block: ContextBlock {
                            name: file_name,
                            priority,
                            content_type: ContentType::Text,
                            content: String::new(),
                            token_estimate: 0,
                            truncatable: true,
                        },
                        source,
                        original_path: Some(path.to_path_buf()),
                        omitted: true,
                        omitted_reason: Some(reason),
                    });
                    return Ok(());
                }
            };
            let token_est = estimate_tokens(&content);
            let block = ContextBlock {
                name: file_name,
                priority,
                content_type: ContentType::Text,
                content,
                token_estimate: token_est,
                truncatable: true,
            };
            self.blocks.push(PackedBlock {
                block,
                source,
                original_path: Some(path.to_path_buf()),
                omitted: false,
                omitted_reason: None,
            });
        }

        Ok(())
    }

    /// Recursively scan a directory, adding all non-excluded files.
    pub fn add_directory(
        &mut self,
        dir_path: &Path,
        source: ContextSource,
        priority: ContextPriority,
        recursive: bool,
    ) -> std::io::Result<()> {
        if self.should_exclude(dir_path) {
            return Ok(());
        }

        if dir_path.is_dir() {
            for entry in fs::read_dir(dir_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if recursive {
                        self.add_directory(&path, source.clone(), priority, true)?;
                    }
                } else {
                    self.add_file(&path, source.clone(), priority)?;
                }
            }
        }
        Ok(())
    }

    /// Add a beautiful structured graphical repository tree as a context block.
    pub fn add_repo_tree(&mut self, root_path: &Path) -> std::io::Result<()> {
        if self.should_exclude(root_path) {
            return Ok(());
        }

        let mut tree_repr = String::new();
        tree_repr.push_str(&format!(
            "[Workspace: {}]\n",
            root_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("root")
        ));

        // Helper closures for clean tree walking
        let builder_clone = self.clone();
        let tree_content =
            generate_tree_repr(root_path, "", &move |p| builder_clone.should_exclude(p))?;
        tree_repr.push_str(&tree_content);

        let tokens = estimate_tokens(&tree_repr);
        let block = ContextBlock {
            name: "repo_tree".to_string(),
            priority: ContextPriority::P2,
            content_type: ContentType::Text,
            content: tree_repr,
            token_estimate: tokens,
            truncatable: true,
        };

        self.blocks.push(PackedBlock {
            block,
            source: ContextSource::RepoTree,
            original_path: Some(root_path.to_path_buf()),
            omitted: false,
            omitted_reason: None,
        });

        Ok(())
    }

    /// Scan workspace root for package manifests (Cargo.toml, package.json)
    /// and pack them to enrich metadata.
    pub fn add_repo_manifest(&mut self, root_path: &Path) -> std::io::Result<()> {
        let manifest_files = vec!["Cargo.toml", "package.json"];
        for manifest in manifest_files {
            let path = root_path.join(manifest);
            if path.exists() {
                self.add_file(&path, ContextSource::RepoTree, ContextPriority::P2)?;
            }
        }
        Ok(())
    }

    /// Consume builder and compute final long context pack package.
    pub fn build(self) -> LongContextPack {
        let total_tokens = self.blocks.iter().map(|pb| pb.block.token_estimate).sum();

        LongContextPack {
            blocks: self.blocks,
            total_token_estimate: total_tokens,
        }
    }
}

/// Helper method to build standard ascii trees.
fn generate_tree_repr(
    dir: &Path,
    prefix: &str,
    exclude_fn: &impl Fn(&Path) -> bool,
) -> std::io::Result<String> {
    let mut result = String::new();
    if dir.is_dir() {
        let entries = fs::read_dir(dir)?;
        let mut paths: Vec<_> = entries
            .filter_map(|e| e.ok().map(|x| x.path()))
            .filter(|p| !exclude_fn(p))
            .collect();
        paths.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        for (idx, path) in paths.iter().enumerate() {
            let is_last = idx == paths.len() - 1;
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            let marker = if is_last { "└── " } else { "├── " };
            result.push_str(&format!("{}{}{}\n", prefix, marker, file_name));

            if path.is_dir() {
                let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
                result.push_str(&generate_tree_repr(path, &new_prefix, exclude_fn)?);
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempTestDir {
        path: PathBuf,
    }

    impl TempTestDir {
        fn new(name: &str) -> Self {
            let path = PathBuf::from(format!(
                "test_temp_{}_{}",
                name,
                uuid::Uuid::new_v4().simple()
            ));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TempTestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn test_builder_excludes() {
        let builder = LongContextPackBuilder::new();

        // 1. Verify VCS, packages, and build directories are excluded (component exact)
        assert!(builder.should_exclude(Path::new("project/.git/config")));
        assert!(builder.should_exclude(Path::new("project/node_modules/lodash/index.js")));
        assert!(builder.should_exclude(Path::new("project/target/debug/app")));
        assert!(builder.should_exclude(Path::new("project/dist/index.js")));
        assert!(builder.should_exclude(Path::new("project/coverage/lcov.info")));

        // 2. Verify binary extensions are excluded
        assert!(builder.should_exclude(Path::new("logo.png")));
        assert!(builder.should_exclude(Path::new("background.jpg")));
        assert!(builder.should_exclude(Path::new("bundle.zip")));
        assert!(builder.should_exclude(Path::new("archive.tar.gz")));

        // 3. Normal source code files are included
        assert!(!builder.should_exclude(Path::new("src/main.rs")));
        assert!(!builder.should_exclude(Path::new("index.html")));

        // 4. Exact matching component check (no false positives on sub-substrings)
        assert!(!builder.should_exclude(Path::new("src/build_script.rs")));
        assert!(!builder.should_exclude(Path::new("docs/targeted-plan.md")));
        assert!(!builder.should_exclude(Path::new(".gitignore")));
    }

    #[test]
    fn test_source_encoding_trace() {
        let temp_dir = TempTestDir::new("source_trace");
        let path = temp_dir.path.join("trace.txt");
        fs::write(&path, "trace contents").unwrap();

        let mut builder = LongContextPackBuilder::new();
        builder
            .add_file(
                &path,
                ContextSource::ActiveFile("trace.txt".to_string()),
                ContextPriority::P1,
            )
            .unwrap();

        let pack = builder.build();
        let flat = pack.to_context_blocks();
        assert_eq!(flat.len(), 1);
        let block_name = &flat[0].name;
        // Verify source info is correctly encoded into the name field
        assert!(block_name.contains("active_file:trace.txt"));
    }

    #[test]
    fn test_large_file_skip_policy() {
        let temp_dir = TempTestDir::new("large_skip");
        let large_file_path = temp_dir.path.join("heavy.log");
        let content = "line\n".repeat(1000); // multiple kilobytes
        fs::write(&large_file_path, content).unwrap();

        let mut builder = LongContextPackBuilder::new()
            .max_file_size(200) // very small max threshold
            .large_file_policy(LargeFilePolicy::Skip);

        builder
            .add_file(
                &large_file_path,
                ContextSource::ActiveFile("heavy.log".to_string()),
                ContextPriority::P1,
            )
            .unwrap();

        let pack = builder.build();
        assert_eq!(pack.blocks.len(), 1);
        let block = &pack.blocks[0];
        assert!(block.omitted);
        assert!(block.omitted_reason.is_some());
        assert!(block.block.content.is_empty());

        let flat = pack.to_context_blocks();
        assert_eq!(flat.len(), 1);
        assert!(flat[0].content.contains("OMITTED"));
    }

    #[test]
    fn test_large_file_head_tail_policy() {
        let temp_dir = TempTestDir::new("head_tail");
        let large_file_path = temp_dir.path.join("moderate.txt");

        let mut lines = Vec::new();
        for i in 1..=200 {
            lines.push(format!("line {}", i));
        }
        fs::write(&large_file_path, lines.join("\n")).unwrap();

        let mut builder = LongContextPackBuilder::new()
            .max_file_size(500)
            .large_file_policy(LargeFilePolicy::HeadTail { lines: 10 });

        builder
            .add_file(
                &large_file_path,
                ContextSource::ActiveFile("moderate.txt".to_string()),
                ContextPriority::P1,
            )
            .unwrap();

        let pack = builder.build();
        assert_eq!(pack.blocks.len(), 1);
        let block = &pack.blocks[0];
        assert!(!block.omitted);
        assert!(block.block.content.contains("TRUNCATED"));
        assert!(block.block.content.contains("line 1"));
        assert!(block.block.content.contains("line 200"));
    }

    #[test]
    fn test_repo_tree_generation() {
        let temp_dir = TempTestDir::new("repo_tree");
        let root = &temp_dir.path;

        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir(root.join(".git")).unwrap(); // should be excluded
        fs::write(root.join("src").join("lib.rs"), "pub fn test() {}").unwrap();
        fs::write(root.join("Cargo.toml"), "[package]").unwrap();
        fs::write(root.join(".git").join("config"), "vcs").unwrap();

        let mut builder = LongContextPackBuilder::new();
        builder.add_repo_tree(root).unwrap();
        builder.add_repo_manifest(root).unwrap();

        let pack = builder.build();
        // Pack should contain tree block + Cargo.toml block
        assert!(pack.blocks.len() >= 1);

        let tree_block = pack
            .blocks
            .iter()
            .find(|b| b.block.name == "repo_tree")
            .unwrap();
        assert!(!tree_block.block.content.contains(".git"));
        assert!(tree_block.block.content.contains("src"));
        assert!(tree_block.block.content.contains("Cargo.toml"));
    }
}
