//! Git CLI工具 - 使用std::process::Command替代git2-rs
//! DEBT-GIT-CLI-W11清偿实现

use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GitCli {
    repo_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GitError {
    CommandFailed { exit_code: i32, stderr: String },
    InvalidRepo,
    IoError(String),
    Utf8Error(String),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::CommandFailed { exit_code, stderr } => write!(f, "Git failed (exit: {}): {}", exit_code, stderr),
            GitError::InvalidRepo => write!(f, "Invalid git repository"),
            GitError::IoError(msg) => write!(f, "IO error: {}", msg),
            GitError::Utf8Error(msg) => write!(f, "UTF-8 error: {}", msg),
        }
    }
}

impl std::error::Error for GitError {}

impl From<std::io::Error> for GitError {
    fn from(err: std::io::Error) -> Self {
        GitError::IoError(err.to_string())
    }
}

impl GitCli {
    pub fn new(repo_path: &str) -> Result<Self, GitError> {
        let path = PathBuf::from(repo_path);
        if !path.exists() || !path.join(".git").exists() {
            return Err(GitError::InvalidRepo);
        }
        Ok(Self { repo_path: path })
    }

    pub fn exec(&self, args: &[&str]) -> Result<String, GitError> {
        let output = Command::new("git").args(args).current_dir(&self.repo_path).output()
            .map_err(|e| GitError::IoError(e.to_string()))?;
        if !output.status.success() {
            return Err(GitError::CommandFailed {
                exit_code: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
        String::from_utf8(output.stdout).map_err(|e| GitError::Utf8Error(e.to_string()))
    }

    pub fn status(&self) -> Result<String, GitError> { self.exec(&["status", "--short"]) }
    pub fn add(&self, path: &str) -> Result<(), GitError> { self.exec(&["add", path]).map(|_| ()) }
    pub fn commit(&self, message: &str) -> Result<(), GitError> { self.exec(&["commit", "-m", message]).map(|_| ()) }
    pub fn push(&self) -> Result<(), GitError> { self.exec(&["push"]).map(|_| ()) }
    pub fn pull(&self) -> Result<(), GitError> { self.exec(&["pull"]).map(|_| ()) }
    pub fn repo_path(&self) -> &Path { &self.repo_path }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_repo() -> Result<(tempfile::TempDir, GitCli), GitError> {
        let dir = tempfile::tempdir()?;
        let path_buf = dir.path().to_path_buf();
        let _ = Command::new("git").args(["init"]).current_dir(&dir.path()).output()?;
        let _ = Command::new("git").args(["config", "user.email", "t@t.com"]).current_dir(&dir.path()).output();
        let _ = Command::new("git").args(["config", "user.name", "Test"]).current_dir(&dir.path()).output();
        let path_str = path_buf.to_str().ok_or_else(|| GitError::IoError("Invalid path".to_string()))?;
        let cli = GitCli::new(path_str)?;
        Ok((dir, cli))
    }

    #[test]
    fn test_invalid_path() {
        assert!(matches!(GitCli::new("/nonexistent/12345"), Err(GitError::InvalidRepo)));
    }

    #[test]
    fn test_non_git_dir() -> Result<(), GitError> {
        let dir = tempfile::tempdir()?;
        let path_str = dir.path().to_str().ok_or_else(|| GitError::IoError("Invalid path".to_string()))?;
        assert!(matches!(GitCli::new(path_str), Err(GitError::InvalidRepo)));
        Ok(())
    }

    #[test]
    fn test_valid_repo() -> Result<(), GitError> {
        let (_temp, cli) = temp_repo()?;
        assert!(cli.repo_path().exists());
        Ok(())
    }

    #[test]
    fn test_status() -> Result<(), GitError> {
        let (_temp, cli) = temp_repo()?;
        let _ = cli.status()?;
        Ok(())
    }

    #[test]
    fn test_add_commit() -> Result<(), GitError> {
        let (temp, cli) = temp_repo()?;
        fs::write(temp.path().join("a.txt"), "hello")?;
        cli.add("a.txt")?;
        cli.commit("init")?;
        let status = cli.status()?;
        assert!(!status.contains("a.txt"));
        Ok(())
    }

    #[test]
    fn test_exec_error() -> Result<(), GitError> {
        let (_temp, cli) = temp_repo()?;
        assert!(matches!(cli.exec(&["invalid-cmd"]), Err(GitError::CommandFailed { .. })));
        Ok(())
    }

    #[test]
    fn test_error_display() {
        assert!(GitError::InvalidRepo.to_string().contains("Invalid"));
        let err = GitError::CommandFailed { exit_code: 1, stderr: "err".to_string() };
        assert!(err.to_string().contains("exit: 1"));
    }
}
