//! 安全审批系统 - 轻量级移植自Codex
//! 参考: codex-twist/codex-rs/core/src/approvals.rs

/// 审批策略 - 控制AI执行权限（5级完整模式，向后兼容）
#[derive(Clone, Debug, PartialEq)]
pub enum ApprovalPolicy {
    AskBeforeExec,  // 每次执行前询问
    AskForDangerous, // 仅危险操作询问（如rm、sudo）
    AskOnceThenAuto, // 首次询问，后续自动
    FullAuto,       // 完全自动（不询问）
    FullDeny,       // 完全拒绝（沙箱模式）
}

/// 审批策略 - 3级精简模式（新增推荐API）
#[derive(Clone, Debug, PartialEq)]
pub enum ApprovalPolicyLevel {
    Ask,  // 询问模式（合并AskBeforeExec+AskForDangerous）
    Auto, // 自动模式（合并AskOnceThenAuto+FullAuto）
    Deny, // 拒绝模式（等同于FullDeny）
}

impl ApprovalPolicyLevel {
    /// 转换为完整5级策略
    pub fn to_policy(&self) -> ApprovalPolicy {
        match self {
            Self::Ask => ApprovalPolicy::AskForDangerous,
            Self::Auto => ApprovalPolicy::FullAuto,
            Self::Deny => ApprovalPolicy::FullDeny,
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ApprovalPolicy {
    fn default() -> Self { ApprovalPolicy::AskBeforeExec }
}

impl ApprovalPolicy {
    /// 检查命令是否需要审批
    pub fn needs_approval(&self, command: &str) -> bool {
        match self {
            ApprovalPolicy::AskBeforeExec => true,
            ApprovalPolicy::AskForDangerous => is_dangerous_command(command),
            ApprovalPolicy::AskOnceThenAuto => false,
            ApprovalPolicy::FullAuto => false,
            ApprovalPolicy::FullDeny => true,
        }
    }

    /// 获取策略描述
    pub fn description(&self) -> &'static str {
        match self {
            ApprovalPolicy::AskBeforeExec => "每次执行前询问确认",
            ApprovalPolicy::AskForDangerous => "仅危险操作询问",
            ApprovalPolicy::AskOnceThenAuto => "首次询问，后续自动",
            ApprovalPolicy::FullAuto => "完全自动执行",
            ApprovalPolicy::FullDeny => "完全拒绝执行",
        }
    }
}

/// 危险命令检测
fn is_dangerous_command(command: &str) -> bool {
    let dangerous_patterns = [
        "rm -rf", "rm -r /", "sudo", "chmod 777", "> /dev", "mkfs", "dd if=",
        ":(){ :|:& };:", // Fork bomb
        "curl | sh", "wget | sh", // Pipe to shell
    ];
    let lower_cmd = command.to_lowercase();
    dangerous_patterns.iter().any(|&p| lower_cmd.contains(p))
}

/// 审批请求
#[derive(Clone, Debug)]
pub struct ApprovalRequest {
    pub id: String,
    pub command: String,
    pub description: String,
    pub risk_level: RiskLevel,
}

/// 风险等级
#[derive(Clone, Debug, PartialEq)]
pub enum RiskLevel { Low, Medium, High, Critical }

impl ApprovalRequest {
    /// 创建新审批请求
    pub fn new(command: String) -> Self {
        Self {
            id: generate_request_id(),
            description: describe_command(&command),
            risk_level: assess_risk(&command),
            command,
        }
    }

    /// 格式化为用户提示
    pub fn prompt(&self) -> String {
        format!(
            "🔒 需要审批\n\n命令: {}\n描述: {}\n风险: {}\n\n是否允许执行? (y/n)",
            self.command, self.description, risk_emoji(&self.risk_level)
        )
    }
}

/// 评估命令风险
fn assess_risk(command: &str) -> RiskLevel {
    let lower = command.to_lowercase();
    if lower.contains("rm -rf") || lower.contains("mkfs") || lower.contains("dd if=/dev") {
        RiskLevel::Critical
    } else if lower.contains("sudo") || lower.contains("chmod") {
        RiskLevel::High
    } else if lower.contains(">") || lower.contains("write") || lower.contains("create") {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    }
}

/// 生成命令描述
fn describe_command(command: &str) -> String {
    if command.starts_with("ls") || command.starts_with("dir") { "列出目录内容".to_string() }
    else if command.starts_with("cat") || command.starts_with("type") { "读取文件内容".to_string() }
    else if command.starts_with("rm") { "删除文件或目录".to_string() }
    else if command.starts_with("mkdir") || command.starts_with("md") { "创建目录".to_string() }
    else if command.starts_with("cp") || command.starts_with("copy") { "复制文件".to_string() }
    else if command.starts_with("mv") || command.starts_with("move") { "移动文件".to_string() }
    else if command.starts_with("git") { "Git版本控制操作".to_string() }
    else if command.starts_with("cargo") { "Rust项目构建".to_string() }
    else if command.starts_with("npm") || command.starts_with("yarn") { "Node.js包管理".to_string() }
    else if command.starts_with("docker") { "Docker容器操作".to_string() }
    else { "执行系统命令".to_string() }
}

/// 风险等级表情
fn risk_emoji(level: &RiskLevel) -> &'static str {
    match level {
        RiskLevel::Low => "🟢 低",
        RiskLevel::Medium => "🟡 中",
        RiskLevel::High => "🟠 高",
        RiskLevel::Critical => "🔴 极高",
    }
}

/// 生成请求ID
fn generate_request_id() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("req-{}", ts)
}

/// 审批结果
#[derive(Clone, Debug)]
pub enum ApprovalResult {
    Approved,
    Denied { reason: String },
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_policy() {
        let policy = ApprovalPolicy::AskBeforeExec;
        assert!(policy.needs_approval("ls -la"));
        let policy = ApprovalPolicy::FullAuto;
        assert!(!policy.needs_approval("ls -la"));
    }

    #[test]
    fn test_dangerous_command() {
        assert!(is_dangerous_command("rm -rf /"));
        assert!(is_dangerous_command("sudo apt install"));
        assert!(!is_dangerous_command("ls -la"));
    }

    #[test]
    fn test_risk_assessment() {
        let req = ApprovalRequest::new("rm -rf /tmp/*".to_string());
        assert!(matches!(req.risk_level, RiskLevel::Critical));
        let req = ApprovalRequest::new("ls -la".to_string());
        assert!(matches!(req.risk_level, RiskLevel::Low));
    }

    #[test]
    fn test_approval_policy_level() {
        assert_eq!(ApprovalPolicyLevel::Ask.to_policy(), ApprovalPolicy::AskForDangerous);
        assert_eq!(ApprovalPolicyLevel::Auto.to_policy(), ApprovalPolicy::FullAuto);
        assert_eq!(ApprovalPolicyLevel::Deny.to_policy(), ApprovalPolicy::FullDeny);
    }
}
