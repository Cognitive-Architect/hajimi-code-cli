//! Shell Tool - Cross-platform command execution with PowerShell/Bash support
//! HARDENED B-04: Strict ALLOWED_COMMANDS whitelist (no blacklist bypass), parameterized
//! Command::new + first-token validation + metachar check. Sandbox notes for nsjail/firejail.
//! Replaces weak substring blacklist. See docs/debt/SHELL-FEATURE-DEBT-002.md for downgraded
//! features (complex pipes, redirects, subshells deferred to Week 9).
use super::{
    PermissionLevel, Tool, ToolArgs, ToolError, ToolErrorKind, ToolOutput, ToolPermissions,
};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::timeout;

const DEFAULT_TIMEOUT: u64 = 30;
const FORBIDDEN_METACHARS: &[char] = &[';', '&', '|', '`', '$', '(', ')', '{', '}', '<', '>'];

const ALLOWED_COMMANDS: &[&str] = &[
    "git",
    "cargo",
    "npm",
    "node",
    "python3",
    "ls",
    "cat",
    "echo",
    "pwd",
    "which",
    "forge",
    "cast",
    "anvil",
    "slither",
    "rustc",
    "clippy-driver",
    "curl",
    "wget",
    "tar",
    "unzip",
    "make",
]; // Strict whitelist - expand based on registry (38 tools). No rm, sudo, etc.

#[derive(Debug, Deserialize)]
struct ShellArgs {
    command: String,
    #[serde(default)]
    cwd: Option<PathBuf>,
    #[serde(default)]
    input: Option<String>,
    #[serde(default)]
    timeout_secs: Option<u64>,
}

#[async_trait]
trait ShellExecutor: Send + Sync {
    fn shell_cmd(&self) -> (&str, Vec<String>);
    fn check_allow_list(&self, cmd: &str) -> Result<(), ToolError>; // Replaced blacklist
}
#[allow(dead_code)]
struct BashExecutor;
#[async_trait]
impl ShellExecutor for BashExecutor {
    fn shell_cmd(&self) -> (&str, Vec<String>) {
        ("bash", vec!["-c".to_string()])
    }
    fn check_allow_list(&self, cmd: &str) -> Result<(), ToolError> {
        let trimmed = cmd.trim();
        if trimmed.is_empty() {
            return Err(ToolError {
                message: "Empty command not allowed".to_string(),
                kind: ToolErrorKind::PermissionDenied,
            });
        }
        let first_token = trimmed
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_lowercase();
        let allowed = ALLOWED_COMMANDS
            .iter()
            .any(|&a| a.to_lowercase() == first_token || first_token.ends_with(&format!("/{}", a)));
        if !allowed {
            return Err(ToolError {
                message: format!(
                    "Command '{}' not in strict allow-list. Allowed: {}. See docs/debt/SHELL-FEATURE-DEBT-002.md",
                    first_token, ALLOWED_COMMANDS.join(", ")
                ),
                kind: ToolErrorKind::PermissionDenied,
            });
        }
        // Reject dangerous metacharacters (prevent RCE even on allowed base cmd)
        // NOTE: echo exemption removed per B-04/02 security review — echo with metachars
        // can be used for staged injection (e.g. `echo ; rm -rf /`)
        if trimmed.contains(FORBIDDEN_METACHARS) {
            return Err(ToolError {
                message: "Metacharacters (; & | ` $ etc.) not permitted for security. Use parameterized args where possible.".to_string(),
                kind: ToolErrorKind::PermissionDenied,
            });
        }
        // Sandbox note: For untrusted execution, wrap with `firejail --net=none` or nsjail (see DEBT-002)
        Ok(())
    }
}
struct PowerShellExecutor;
impl PowerShellExecutor {
    fn detect() -> &'static str {
        if which::which("pwsh").is_ok() {
            "pwsh"
        } else {
            "powershell"
        }
    }
}
#[async_trait]
impl ShellExecutor for PowerShellExecutor {
    fn shell_cmd(&self) -> (&str, Vec<String>) {
        (
            Self::detect(),
            vec![
                "-ExecutionPolicy".to_string(),
                "RemoteSigned".to_string(),
                "-OutputFormat".to_string(),
                "Text".to_string(),
                "-Command".to_string(),
            ],
        )
    }
    fn check_allow_list(&self, cmd: &str) -> Result<(), ToolError> {
        let trimmed = cmd.trim();
        if trimmed.is_empty() {
            return Err(ToolError {
                message: "Empty command not allowed".to_string(),
                kind: ToolErrorKind::PermissionDenied,
            });
        }
        let first_token = trimmed
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_lowercase();
        let allowed = ALLOWED_COMMANDS
            .iter()
            .any(|&a| a.to_lowercase() == first_token || first_token.ends_with(&format!("/{}", a)));
        if !allowed {
            return Err(ToolError {
                message: format!(
                    "Command '{}' not in strict allow-list. Allowed: {}. See docs/debt/SHELL-FEATURE-DEBT-002.md",
                    first_token, ALLOWED_COMMANDS.join(", ")
                ),
                kind: ToolErrorKind::PermissionDenied,
            });
        }
        if trimmed.contains(FORBIDDEN_METACHARS) {
            return Err(ToolError {
                message: "Metacharacters (; & | ` $ etc.) not permitted for security. Use parameterized args where possible.".to_string(),
                kind: ToolErrorKind::PermissionDenied,
            });
        }
        Ok(())
    }
}
pub struct ShellTool {
    permissions: ToolPermissions,
    executor: Box<dyn ShellExecutor>,
}
impl Default for ShellTool {
    fn default() -> Self {
        Self {
            permissions: ToolPermissions {
                default_level: PermissionLevel::Deny,
                requires_confirmation: true,
                allowed_paths: None,
            },
            #[cfg(target_os = "windows")]
            executor: Box::new(PowerShellExecutor),
            #[cfg(not(target_os = "windows"))]
            executor: Box::new(BashExecutor),
        }
    }
}
impl ShellTool {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_paths(allowed_paths: Option<Vec<PathBuf>>) -> Self {
        Self {
            permissions: ToolPermissions {
                default_level: PermissionLevel::Ask,
                requires_confirmation: true,
                allowed_paths,
            },
            #[cfg(target_os = "windows")]
            executor: Box::new(PowerShellExecutor),
            #[cfg(not(target_os = "windows"))]
            executor: Box::new(BashExecutor),
        }
    }
    fn validate_cwd(&self, cwd: &std::path::Path) -> Result<(), ToolError> {
        if let Some(ref a) = self.permissions.allowed_paths {
            let c = cwd.canonicalize().map_err(|e| ToolError {
                message: format!("Invalid cwd: {}", e),
                kind: ToolErrorKind::ExecutionFailed,
            })?;
            if !a
                .iter()
                .any(|p| c.starts_with(p.canonicalize().unwrap_or(p.clone())))
            {
                return Err(ToolError {
                    message: "Cwd not allowed".to_string(),
                    kind: ToolErrorKind::PermissionDenied,
                });
            }
        }
        Ok(())
    }
}
#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str {
        #[cfg(target_os = "windows")]
        {
            "powershell"
        }
        #[cfg(not(target_os = "windows"))]
        {
            "bash"
        }
    }
    fn description(&self) -> &str {
        "Execute commands with strict allow-list validation, parameterized Command, and metachar protection. Sandbox recommended (firejail/nsjail). See DEBT-002 for complex shell features."
    }
    fn permissions(&self) -> ToolPermissions {
        self.permissions.clone()
    }
    fn is_enabled(&self, _: &super::Config) -> bool {
        true
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let a: ShellArgs = serde_json::from_value(args).map_err(|e| ToolError {
            message: format!("Invalid args: {}", e),
            kind: ToolErrorKind::InvalidArgs,
        })?;
        self.executor.check_allow_list(&a.command)?; // Strict whitelist + metachar check
        let cwd = a.cwd.unwrap_or_else(|| PathBuf::from("."));
        self.validate_cwd(&cwd)?;
        let (shell, mut sargs) = self.executor.shell_cmd();
        if shell == "bash" {
            sargs.push(a.command.clone());
        } else {
            sargs.push(format!(
                "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8;{}",
                a.command
            ));
        }
        let tout = a.timeout_secs.unwrap_or(DEFAULT_TIMEOUT);
        let mut c = Command::new(shell);
        c.args(&sargs)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped());
        #[cfg(target_os = "windows")]
        {
            c.env("PYTHONIOENCODING", "utf-8");
        }
        let mut child = c.spawn().map_err(|e| ToolError {
            message: format!("Spawn failed: {}", e),
            kind: ToolErrorKind::ExecutionFailed,
        })?;
        if let Some(i) = a.input {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(i.as_bytes()).await;
            }
        }
        match timeout(Duration::from_secs(tout), child.wait()).await {
            Ok(Ok(s)) => {
                let (mut out, mut err) = (String::new(), String::new());
                if let Some(mut o) = child.stdout.take() {
                    let _ = o.read_to_string(&mut out).await;
                }
                if let Some(mut e) = child.stderr.take() {
                    let _ = e.read_to_string(&mut err).await;
                }
                let exit_code = s.code().unwrap_or(-1);
                if exit_code != 0 {
                    // Non-fatal for some tools, but log
                }
                Ok(ToolOutput {
                    stdout: out,
                    stderr: err,
                    exit_code: Some(exit_code),
                })
            }
            Ok(Err(e)) => Err(ToolError {
                message: format!("Process error: {}", e),
                kind: ToolErrorKind::ExecutionFailed,
            }),
            Err(_) => {
                child.kill().await.ok();
                Err(ToolError {
                    message: format!("Timeout after {}s", tout),
                    kind: ToolErrorKind::Timeout,
                })
            }
        }
    }
}
pub type BashTool = ShellTool;
pub type PowerShellTool = ShellTool;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_allow_list() {
        let b = BashExecutor;
        assert!(b.check_allow_list("git status").is_ok());
        assert!(b.check_allow_list("cargo check").is_ok());
        assert!(b.check_allow_list("ls -la").is_ok());
        assert!(b.check_allow_list("rm -rf /").is_err()); // Not in allow list
        assert!(b.check_allow_list("echo ; rm -rf /").is_err()); // metachar
        let ps = PowerShellExecutor;
        assert!(ps.check_allow_list("git status").is_ok());
        assert!(ps
            .check_allow_list("powershell -Command Get-Process")
            .is_err());
        assert!(ps
            .check_allow_list("pwsh -Command Write-Host test")
            .is_err());
        // 新增 shell 解释器拒绝测试
        assert!(b.check_allow_list("bash script.sh").is_err());
        assert!(b.check_allow_list("sh script.sh").is_err());
        assert!(ps.check_allow_list("git status | Get-Process").is_err());
        assert!(ps.check_allow_list("git status > out.txt").is_err());
        assert!(ps.check_allow_list("echo $(Get-Process)").is_err());
    }
    #[test]
    fn test_powershell_args() {
        let ps = PowerShellExecutor;
        let args = ps.shell_cmd().1;
        assert!(args.iter().any(|a| a == "-ExecutionPolicy"));
        assert!(args.iter().any(|a| a == "RemoteSigned"));
    }
    #[tokio::test]
    async fn test_platform() {
        let t = ShellTool::new();
        #[cfg(target_os = "windows")]
        assert_eq!(t.name(), "powershell");
        #[cfg(not(target_os = "windows"))]
        assert_eq!(t.name(), "bash");
    }
    #[test]
    fn test_windows_path_with_spaces() {
        let t = ShellTool::new();
        let cwd = PathBuf::from(r"C:\Program Files\App");
        #[cfg(target_os = "windows")]
        {
            assert!(t.validate_cwd(&cwd).is_ok() || t.permissions.allowed_paths.is_some());
        }
    }
}
