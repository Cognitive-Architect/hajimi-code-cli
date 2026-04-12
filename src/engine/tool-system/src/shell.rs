//! Shell Tool - Cross-platform command execution with PowerShell/Bash support
use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::timeout;
use super::{PermissionLevel, Tool, ToolArgs, ToolError, ToolErrorKind, ToolOutput, ToolPermissions};
const DEFAULT_TIMEOUT: u64 = 30;
#[cfg(target_os = "windows")]
const BLACKLIST: &[&str] = &["rm -rf /", "format", "del /f /s /q c:\\", "Remove-Item -Recurse -Force C:\\"];
#[cfg(not(target_os = "windows"))]
const BLACKLIST: &[&str] = &["rm -rf /", ":(){ :|:& };:", "mkfs", "dd if=/dev/zero", "> /dev/sda"];
#[derive(Debug, Deserialize)]
struct ShellArgs { command: String, #[serde(default)] cwd: Option<PathBuf>, #[serde(default)] input: Option<String>, #[serde(default)] timeout_secs: Option<u64> }
#[async_trait]
trait ShellExecutor: Send + Sync {
    fn shell_cmd(&self) -> (&str, Vec<String>);
    fn check_blacklist(&self, cmd: &str) -> Result<(), ToolError>;
}
struct BashExecutor;
#[async_trait]
impl ShellExecutor for BashExecutor {
    fn shell_cmd(&self) -> (&str, Vec<String>) { ("bash", vec!["-c".to_string()]) }
    fn check_blacklist(&self, cmd: &str) -> Result<(), ToolError> {
        let n = cmd.to_lowercase();
        for p in BLACKLIST { if n.contains(&p.to_lowercase()) { return Err(ToolError { message: format!("Forbidden: {}", p), kind: ToolErrorKind::PermissionDenied }); } }
        Ok(())
    }
}
struct PowerShellExecutor;
impl PowerShellExecutor {
    fn detect() -> &'static str { if which::which("pwsh").is_ok() { "pwsh" } else { "powershell" } }
}
#[async_trait]
impl ShellExecutor for PowerShellExecutor {
    fn shell_cmd(&self) -> (&str, Vec<String>) {
        (Self::detect(), vec!["-ExecutionPolicy".to_string(), "Bypass".to_string(), "-OutputFormat".to_string(), "Text".to_string(), "-Command".to_string()])
    }
    fn check_blacklist(&self, cmd: &str) -> Result<(), ToolError> {
        let n = cmd.to_lowercase();
        for p in BLACKLIST { if n.contains(&p.to_lowercase()) { return Err(ToolError { message: format!("Forbidden: {}", p), kind: ToolErrorKind::PermissionDenied }); } }
        Ok(())
    }
}
pub struct ShellTool { permissions: ToolPermissions, executor: Box<dyn ShellExecutor> }
impl Default for ShellTool {
    fn default() -> Self {
        Self { permissions: ToolPermissions { default_level: PermissionLevel::Deny, requires_confirmation: true, allowed_paths: None },
            #[cfg(target_os = "windows")] executor: Box::new(PowerShellExecutor),
            #[cfg(not(target_os = "windows"))] executor: Box::new(BashExecutor) }
    }
}
impl ShellTool {
    pub fn new() -> Self { Self::default() }
    pub fn with_paths(allowed_paths: Option<Vec<PathBuf>>) -> Self {
        Self { permissions: ToolPermissions { default_level: PermissionLevel::Ask, requires_confirmation: true, allowed_paths },
            #[cfg(target_os = "windows")] executor: Box::new(PowerShellExecutor),
            #[cfg(not(target_os = "windows"))] executor: Box::new(BashExecutor) }
    }
    fn validate_cwd(&self, cwd: &PathBuf) -> Result<(), ToolError> {
        if let Some(ref a) = self.permissions.allowed_paths {
            let c = cwd.canonicalize().map_err(|e| ToolError { message: format!("Invalid cwd: {}", e), kind: ToolErrorKind::ExecutionFailed })?;
            if !a.iter().any(|p| c.starts_with(p.canonicalize().unwrap_or(p.clone()))) {
                return Err(ToolError { message: "Cwd not allowed".to_string(), kind: ToolErrorKind::PermissionDenied });
            }
        }
        Ok(())
    }
}
#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str { #[cfg(target_os = "windows")] { "powershell" } #[cfg(not(target_os = "windows"))] { "bash" } }
    fn description(&self) -> &str { #[cfg(target_os = "windows")] { "Execute PowerShell with security" } #[cfg(not(target_os = "windows"))] { "Execute shell with security" } }
    fn permissions(&self) -> ToolPermissions { self.permissions.clone() }
    fn is_enabled(&self, _: &super::Config) -> bool { true }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let a: ShellArgs = serde_json::from_value(args).map_err(|e| ToolError { message: format!("Invalid args: {}", e), kind: ToolErrorKind::InvalidArgs })?;
        self.executor.check_blacklist(&a.command)?;
        let cwd = a.cwd.unwrap_or_else(|| PathBuf::from("."));
        self.validate_cwd(&cwd)?;
        let (shell, mut sargs) = self.executor.shell_cmd();
        if shell == "bash" { sargs.push(a.command.clone()); } else { sargs.push(format!("[Console]::OutputEncoding=[System.Text.Encoding]::UTF8;{}", a.command)); }
        let tout = a.timeout_secs.unwrap_or(DEFAULT_TIMEOUT);
        let mut c = Command::new(shell);
        c.args(&sargs).current_dir(&cwd).stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::piped());
        #[cfg(target_os = "windows")] { c.env("PYTHONIOENCODING", "utf-8"); }
        let mut child = c.spawn().map_err(|e| ToolError { message: format!("Spawn failed: {}", e), kind: ToolErrorKind::ExecutionFailed })?;
        if let Some(i) = a.input { if let Some(mut stdin) = child.stdin.take() { let _ = stdin.write_all(i.as_bytes()).await; } }
        match timeout(Duration::from_secs(tout), child.wait()).await {
            Ok(Ok(s)) => {
                let (mut out, mut err) = (String::new(), String::new());
                if let Some(mut o) = child.stdout.take() { let _ = o.read_to_string(&mut out).await; }
                if let Some(mut e) = child.stderr.take() { let _ = e.read_to_string(&mut err).await; }
                Ok(ToolOutput { stdout: out, stderr: err, exit_code: Some(s.code().unwrap_or(-1)) })
            }
            Ok(Err(e)) => Err(ToolError { message: format!("Process error: {}", e), kind: ToolErrorKind::ExecutionFailed }),
            Err(_) => { child.kill().await.ok(); Err(ToolError { message: format!("Timeout after {}s", tout), kind: ToolErrorKind::Timeout }) }
        }
    }
}
pub type BashTool = ShellTool;
pub type PowerShellTool = ShellTool;
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn test_powershell_blacklist() { let ps = PowerShellExecutor; assert!(ps.check_blacklist("Remove-Item -Recurse -Force C:\\").is_err()); assert!(ps.check_blacklist("Get-Process").is_ok()); }
    #[test] fn test_bash_blacklist() { let b = BashExecutor; assert!(b.check_blacklist("rm -rf /").is_err()); assert!(b.check_blacklist("ls").is_ok()); }
    #[test] fn test_powershell_args() { let ps = PowerShellExecutor; let args = ps.shell_cmd().1; assert!(args.iter().any(|a| a == "-ExecutionPolicy")); assert!(args.iter().any(|a| a == "Bypass")); }
    #[tokio::test] async fn test_platform() { let t = ShellTool::new(); #[cfg(target_os = "windows")] assert_eq!(t.name(), "powershell"); #[cfg(not(target_os = "windows"))] assert_eq!(t.name(), "bash"); }
    #[test] fn test_windows_path_with_spaces() { let t = ShellTool::new(); let cwd = PathBuf::from(r"C:\Program Files\App"); #[cfg(target_os = "windows")] { assert!(t.validate_cwd(&cwd).is_ok() || t.permissions.allowed_paths.is_some()); } }
}
