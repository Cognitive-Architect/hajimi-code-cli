// Codex CLI 接口定义提取
// 来源: https://github.com/openai/codex
// 日期: 2026-03-04
// 说明: 从Rust源码提取的TypeScript接口定义，用于Twist方案

// ============================================
// 1. Thread 接口定义
// ============================================

/**
 * Thread - 对话线程
 * 对应: codex-rs/core/src/codex.rs
 */
export interface Thread {
  /** 线程唯一ID */
  id: string;
  
  /** 线程标题(可选) */
  title?: string;
  
  /** 回合列表 */
  turns: Turn[];
  
  /** 创建时间 */
  created_at: Date;
  
  /** 最后更新时间 */
  updated_at: Date;
  
  /** 工作目录 */
  cwd: string;
  
  /** 线程元数据 */
  meta: ThreadMeta;
  
  /** 线程来源 */
  source: ThreadSource;
  
  /** 关联的模型配置 */
  model_config: ModelConfig;
}

/**
 * ThreadMeta - 线程元数据
 */
export interface ThreadMeta {
  /** 消息数量 */
  message_count: number;
  
  /** 总token数 */
  total_tokens: number;
  
  /** 当前上下文窗口使用 */
  context_window_usage: number;
  
  /** 是否已归档 */
  archived: boolean;
  
  /** 标签 */
  tags: string[];
}

/**
 * ThreadSource - 线程来源
 */
export enum ThreadSource {
  /** 本地存储 */
  Local = 'local',
  
  /** 云端OpenAI */
  Cloud = 'cloud',
  
  /** 双向同步 */
  Synced = 'synced',
  
  /** LCR存储 (Twist新增) */
  LCR = 'lcr'
}

// ============================================
// 2. Turn 接口定义
// ============================================

/**
 * Turn - 对话回合
 * 对应: codex-rs/protocol/src/protocol.rs
 */
export interface Turn {
  /** 回合唯一ID */
  id: string;
  
  /** 回合序号 */
  sequence: number;
  
  /** 角色 */
  role: Role;
  
  /** 内容 */
  content: ContentItem[];
  
  /** 时间戳 */
  timestamp: Date;
  
  /** 工具调用 */
  tool_calls?: ToolCall[];
  
  /** 工具结果 */
  tool_results?: ToolResult[];
  
  /** 回合元数据 */
  metadata: TurnMetadata;
  
  /** 使用的token数 */
  token_usage?: TokenUsage;
  
  /** 回合状态 */
  status: TurnStatus;
  
  /** 审批记录 */
  approvals?: ApprovalRecord[];
}

/**
 * Role - 对话角色
 */
export enum Role {
  /** 用户 */
  User = 'user',
  
  /** 助手 */
  Assistant = 'assistant',
  
  /** 系统 */
  System = 'system',
  
  /** 开发者 */
  Developer = 'developer',
  
  /** 工具 */
  Tool = 'tool'
}

/**
 * ContentItem - 内容项
 */
export type ContentItem = 
  | TextContent 
  | ImageContent 
  | FileContent
  | ReasoningContent;

export interface TextContent {
  type: 'text';
  text: string;
}

export interface ImageContent {
  type: 'image';
  /** base64编码或URL */
  source: string;
  /** 图片格式 */
  mime_type: string;
}

export interface FileContent {
  type: 'file';
  /** 文件路径 */
  path: string;
  /** 文件内容(可选) */
  content?: string;
}

export interface ReasoningContent {
  type: 'reasoning';
  /** 推理过程 */
  reasoning: string;
}

/**
 * TurnMetadata - 回合元数据
 */
export interface TurnMetadata {
  /** 模型ID */
  model?: string;
  
  /** 温度参数 */
  temperature?: number;
  
  /** 使用的技能 */
  skills_used: string[];
  
  /** 沙箱类型 */
  sandbox_type: SandboxType;
  
  /** 执行时间(ms) */
  execution_time_ms?: number;
  
  /** LCR扩展: 记忆层级 (Twist新增) */
  memory_tier?: MemoryTier;
}

/**
 * MemoryTier - LCR记忆层级 (Twist新增)
 */
export enum MemoryTier {
  /** 焦点记忆 */
  Focus = 'focus',
  
  /** 工作记忆 */
  Working = 'working',
  
  /** 归档记忆 */
  Archive = 'archive',
  
  /** RAG检索 */
  Rag = 'rag'
}

/**
 * TurnStatus - 回合状态
 */
export enum TurnStatus {
  /** 进行中 */
  InProgress = 'in_progress',
  
  /** 已完成 */
  Complete = 'complete',
  
  /** 已中断 */
  Aborted = 'aborted',
  
  /** 错误 */
  Error = 'error'
}

/**
 * TokenUsage - Token使用统计
 */
export interface TokenUsage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

// ============================================
// 3. AuthConfig 接口定义
// ============================================

/**
 * AuthConfig - 认证配置
 * 对应: codex-rs/keyring-store/src/lib.rs
 */
export interface AuthConfig {
  /** 认证方法 */
  method: AuthMethod;
  
  /** 访问Token (可选，零账号模式为空) */
  access_token?: string;
  
  /** 刷新Token (可选) */
  refresh_token?: string;
  
  /** 过期时间 */
  expires_at?: Date;
  
  /** 组织ID (企业版) */
  organization_id?: string;
  
  /** 用户信息 */
  user?: UserInfo;
}

/**
 * AuthMethod - 认证方法
 */
export enum AuthMethod {
  /** ChatGPT OAuth */
  ChatGPTOAuth = 'chatgpt_oauth',
  
  /** API Key */
  ApiKey = 'api_key',
  
  /** 组织认证 */
  Organization = 'organization',
  
  /** 零账号模式 (Twist新增) */
  NoAuth = 'no_auth'
}

export interface UserInfo {
  id: string;
  email: string;
  name?: string;
  plan?: string;
}

// ============================================
// 4. Storage 接口定义
// ============================================

/**
 * ThreadStorage - 线程存储接口
 * 对应: codex-rs/core/src/storage.rs (Twist新增)
 */
export interface ThreadStorage {
  /** 创建新线程 */
  createThread(config: ThreadConfig): Promise<string>;
  
  /** 保存回合 */
  saveTurn(threadId: string, turn: Turn): Promise<void>;
  
  /** 加载线程历史 */
  loadThread(threadId: string): Promise<Turn[]>;
  
  /** 列出所有线程 */
  listThreads(): Promise<ThreadSummary[]>;
  
  /** 删除线程 */
  deleteThread(threadId: string): Promise<void>;
  
  /** 导出线程 */
  exportThread(threadId: string, format: ExportFormat): Promise<Uint8Array>;
  
  /** 搜索线程 */
  searchThreads(query: string): Promise<SearchResult[]>;
  
  /** 创建快照 */
  createSnapshot(threadId: string): Promise<Snapshot>;
  
  /** 从快照恢复 */
  restoreFromSnapshot(snapshotId: string): Promise<Thread>;
}

/**
 * ThreadConfig - 线程配置
 */
export interface ThreadConfig {
  /** 初始工作目录 */
  cwd: string;
  
  /** 初始模型 */
  model: string;
  
  /** 审批策略 */
  approval_policy: ApprovalPolicy;
  
  /** 沙箱策略 */
  sandbox_policy: SandboxPolicy;
  
  /** 初始指令 */
  instructions?: string;
  
  /** MCP服务器配置 */
  mcp_servers?: McpServerConfig[];
}

/**
 * ThreadSummary - 线程摘要
 */
export interface ThreadSummary {
  id: string;
  title?: string;
  created_at: Date;
  updated_at: Date;
  message_count: number;
  source: ThreadSource;
}

/**
 * ExportFormat - 导出格式
 */
export enum ExportFormat {
  HCTX = 'hctx',       // LCR格式
  JSON = 'json',       // JSON格式
  MARKDOWN = 'md',     // Markdown
  ZIP = 'zip'          // 压缩包
}

/**
 * Snapshot - 上下文快照
 */
export interface Snapshot {
  id: string;
  thread_id: string;
  created_at: Date;
  /** 快照大小(bytes) */
  size: number;
  /** 包含的回合数 */
  turn_count: number;
  /** 快照格式版本 */
  version: string;
}

/**
 * SearchResult - 搜索结果
 */
export interface SearchResult {
  thread_id: string;
  turn_id: string;
  /** 匹配内容 */
  snippet: string;
  /** 相关度分数 */
  score: number;
  /** 记忆层级 */
  memory_tier: MemoryTier;
}

// ============================================
// 5. LCR扩展接口 (Twist新增)
// ============================================

/**
 * LcrStorageConfig - LCR存储配置
 */
export interface LcrStorageConfig {
  /** 存储根目录 */
  root_dir: string;
  
  /** HCTX格式版本 */
  hctx_version: number;
  
  /** 启用加密 */
  encryption: boolean;
  
  /** 压缩级别 0-9 */
  compression_level: number;
  
  /** 启用四级存储 */
  enable_tiered: boolean;
  
  /** 启用MemGPT */
  enable_memgpt: boolean;
}

/**
 * HctxHeader - HCTX文件头 (64字节)
 */
export interface HctxHeader {
  /** 魔数 "HCTX" */
  magic: Uint8Array;  // 4 bytes
  
  /** 版本号 */
  version: number;    // 1 byte
  
  /** 线程ID */
  thread_id: string;  // 16 bytes (UUID)
  
  /** 创建时间 */
  created_at: Date;   // 8 bytes (timestamp)
  
  /** 加密标志 */
  encryption_flag: number;  // 1 byte
  
  /** 保留字段 */
  reserved: Uint8Array;     // 40 bytes
}

/**
 * MemoryStatus - 内存状态报告
 */
export interface MemoryStatus {
  focus_usage: number;
  working_usage: number;
  archive_count: number;
  rag_indexed: number;
  predicted_gc_time_ms: number;
}

// ============================================
// 6. 策略与配置接口
// ============================================

/**
 * ApprovalPolicy - 审批策略
 */
export enum ApprovalPolicy {
  /** 始终建议审批 */
  Suggest = 'suggest',
  
  /** 仅失败时审批 */
  OnFailure = 'on_failure',
  
  /** 自动审批安全命令 */
  Auto = 'auto',
  
  /** 永不审批(背景模式) */
  Never = 'never',
  
  /** 只读模式 */
  ReadOnly = 'read_only'
}

/**
 * SandboxPolicy - 沙箱策略
 */
export interface SandboxPolicy {
  /** 沙箱类型 */
  type: SandboxType;
  
  /** 允许的目录 */
  allowed_directories: string[];
  
  /** 禁止的目录 */
  forbidden_directories: string[];
  
  /** 允许的网络主机 */
  network_allowed_hosts: string[];
  
  /** 禁止的网络 */
  network_forbidden: boolean;
  
  /** 允许的环境变量 */
  allowed_env_vars: string[];
  
  /** 最大执行时间 */
  max_execution_time_secs: number;
}

/**
 * SandboxType - 沙箱类型
 */
export enum SandboxType {
  /** 无沙箱 */
  None = 'none',
  
  /** macOS Seatbelt */
  MacosSeatbelt = 'macos_seatbelt',
  
  /** Linux Landlock + seccomp */
  LinuxSeccomp = 'linux_seccomp',
  
  /** Windows沙箱 */
  WindowsSandbox = 'windows_sandbox'
}

/**
 * ModelConfig - 模型配置
 */
export interface ModelConfig {
  /** 模型ID */
  model: string;
  
  /** API基础URL */
  api_base?: string;
  
  /** 温度参数 */
  temperature: number;
  
  /** 最大token数 */
  max_tokens: number;
  
  /** Top P */
  top_p?: number;
  
  /** 频率惩罚 */
  frequency_penalty?: number;
  
  /** 存在惩罚 */
  presence_penalty?: number;
}

/**
 * McpServerConfig - MCP服务器配置
 */
export interface McpServerConfig {
  name: string;
  command: string;
  args?: string[];
  env?: Record<string, string>;
}

// ============================================
// 7. 工具相关接口
// ============================================

/**
 * ToolCall - 工具调用
 */
export interface ToolCall {
  /** 调用ID */
  id: string;
  
  /** 工具类型 */
  type: string;
  
  /** 函数调用 */
  function?: FunctionCall;
  
  /** MCP工具调用 */
  mcp_tool?: McpToolCall;
}

/**
 * FunctionCall - 函数调用
 */
export interface FunctionCall {
  name: string;
  arguments: string;
}

/**
 * McpToolCall - MCP工具调用
 */
export interface McpToolCall {
  server: string;
  tool: string;
  arguments: Record<string, unknown>;
}

/**
 * ToolResult - 工具结果
 */
export interface ToolResult {
  /** 对应调用ID */
  call_id: string;
  
  /** 输出内容 */
  output: string;
  
  /** 退出码 */
  exit_code?: number;
  
  /** 是否成功 */
  success: boolean;
}

/**
 * ApprovalRecord - 审批记录
 */
export interface ApprovalRecord {
  /** 审批ID */
  id: string;
  
  /** 审批类型 */
  type: ApprovalType;
  
  /** 请求内容 */
  request: string;
  
  /** 审批结果 */
  decision: ApprovalDecision;
  
  /** 审批时间 */
  timestamp: Date;
  
  /** 作用域 */
  scope: ApprovalScope;
}

/**
 * ApprovalType - 审批类型
 */
export enum ApprovalType {
  ExecCommand = 'exec_command',
  ApplyPatch = 'apply_patch',
  FileWrite = 'file_write',
  NetworkAccess = 'network_access'
}

/**
 * ApprovalDecision - 审批决定
 */
export enum ApprovalDecision {
  Approve = 'approve',
  Deny = 'deny',
  ApproveOnce = 'approve_once',
  ApproveSession = 'approve_session'
}

/**
 * ApprovalScope - 审批作用域
 */
export enum ApprovalScope {
  Once = 'once',
  Session = 'session',
  Always = 'always'
}

// ============================================
// 8. 协议事件接口
// ============================================

/**
 * EventMsg - 协议事件消息
 * 对应: codex-rs/protocol/src/protocol.rs
 */
export type EventMsg =
  | TurnStartedEvent
  | TurnCompleteEvent
  | TurnAbortedEvent
  | ExecCommandStartEvent
  | ExecCommandEndEvent
  | AgentMessageEvent
  | PatchApplyStartEvent
  | PatchApplyEndEvent;

export interface TurnStartedEvent {
  type: 'turn_started';
  turn_id: string;
  timestamp: Date;
}

export interface TurnCompleteEvent {
  type: 'turn_complete';
  turn_id: string;
  timestamp: Date;
}

export interface TurnAbortedEvent {
  type: 'turn_aborted';
  turn_id: string;
  reason: string;
  timestamp: Date;
}

export interface ExecCommandStartEvent {
  type: 'exec_command_start';
  command: string;
  timestamp: Date;
}

export interface ExecCommandEndEvent {
  type: 'exec_command_end';
  command: string;
  exit_code: number;
  timestamp: Date;
}

export interface AgentMessageEvent {
  type: 'agent_message';
  content: string;
  timestamp: Date;
}

export interface PatchApplyStartEvent {
  type: 'patch_apply_start';
  file: string;
  timestamp: Date;
}

export interface PatchApplyEndEvent {
  type: 'patch_apply_end';
  file: string;
  success: boolean;
  timestamp: Date;
}

// ============================================
// 文档结束
// ============================================

/**
 * 验证信息:
 * - 来源: github.com/openai/codex
 * - Thread/Turn定义: 完整
 * - Storage接口: 包含
 * - LCR扩展: 已标记
 * - approval_policy: 已定义
 */
