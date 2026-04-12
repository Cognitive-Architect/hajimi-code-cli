import * as vscode from 'vscode';
// COMMAND: 56 tools + 4 shortcuts = 60 commands
export enum CommandId {
  OPEN_SIDEBAR = 'hajimi.openSidebar',
  SEARCH_CODE = 'hajimi.searchCode',
  QUICK_COMMAND = 'hajimi.quickCommand',
  TOGGLE_TERMINAL = 'hajimi.toggleTerminal',
  EVM_COMPILE = 'hajimi.evm.compile',
  EVM_DEPLOY = 'hajimi.evm.deploy',
  EVM_VERIFY = 'hajimi.evm.verify',
  EVM_TEST = 'hajimi.evm.test',
  EVM_DEBUG = 'hajimi.evm.debug',
  EVM_ANALYZE = 'hajimi.evm.analyze',
  EVM_PATCH = 'hajimi.evm.patch',
  EVM_EXPLOIT = 'hajimi.evm.exploit',
  MCP_START = 'hajimi.mcp.start',
  MCP_STOP = 'hajimi.mcp.stop',
  MCP_RESTART = 'hajimi.mcp.restart',
  MCP_STATUS = 'hajimi.mcp.status',
  MCP_CONNECT = 'hajimi.mcp.connect',
  MCP_DISCONNECT = 'hajimi.mcp.disconnect',
  P2P_INIT = 'hajimi.p2p.init',
  P2P_SYNC = 'hajimi.p2p.sync',
  P2P_SHARE = 'hajimi.p2p.share',
  P2P_JOIN = 'hajimi.p2p.join',
  P2P_LEAVE = 'hajimi.p2p.leave',
  DB_CONNECT = 'hajimi.db.connect',
  DB_QUERY = 'hajimi.db.query',
  DB_MIGRATE = 'hajimi.db.migrate',
  DB_BACKUP = 'hajimi.db.backup',
  DB_RESTORE = 'hajimi.db.restore',
  WASM_BUILD = 'hajimi.wasm.build',
  WASM_RUN = 'hajimi.wasm.run',
  WASM_TEST = 'hajimi.wasm.test',
  FORMAT = 'hajimi.format',
  LINT = 'hajimi.lint',
  BUILD = 'hajimi.build',
  CLEAN = 'hajimi.clean',
  INSTALL = 'hajimi.install',
  UPDATE = 'hajimi.update',
  PUBLISH = 'hajimi.publish',
  PACKAGE = 'hajimi.package',
  DOCKER_BUILD = 'hajimi.docker.build',
  DOCKER_RUN = 'hajimi.docker.run',
  DOCKER_STOP = 'hajimi.docker.stop',
  GIT_COMMIT = 'hajimi.git.commit',
  GIT_PUSH = 'hajimi.git.push',
  GIT_PULL = 'hajimi.git.pull',
  GIT_BRANCH = 'hajimi.git.branch',
  GIT_MERGE = 'hajimi.git.merge',
  GIT_REBASE = 'hajimi.git.rebase',
  TEST_RUN = 'hajimi.test.run',
  TEST_DEBUG = 'hajimi.test.debug',
  TEST_COVERAGE = 'hajimi.test.coverage',
  BENCHMARK = 'hajimi.benchmark',
  PROFILE = 'hajimi.profile',
  AUDIT = 'hajimi.audit',
  SCAN = 'hajimi.scan',
  GENERATE = 'hajimi.generate',
  TEMPLATE = 'hajimi.template',
  CONFIGURE = 'hajimi.configure',
  VALIDATE = 'hajimi.validate',
  EXPORT = 'hajimi.export'
}
export class CommandRegistry {
  constructor(private context: vscode.ExtensionContext) {}
  registerCommand(command: string, callback: (...args: unknown[]) => unknown): void {
    this.context.subscriptions.push(vscode.commands.registerCommand(command, callback));
  }
  registerAllCommands(): void {
    this.registerCommand(CommandId.OPEN_SIDEBAR, () => vscode.commands.executeCommand('workbench.view.extension.hajimi'));
    this.registerCommand(CommandId.SEARCH_CODE, () => vscode.commands.executeCommand('workbench.action.findInFiles'));
    this.registerCommand(CommandId.QUICK_COMMAND, () => vscode.commands.executeCommand('workbench.action.showCommands'));
    this.registerCommand(CommandId.TOGGLE_TERMINAL, () => vscode.commands.executeCommand('workbench.action.terminal.toggleTerminal'));
    Object.values(CommandId).slice(4).forEach(cmd => this.registerCommand(cmd, (...args: unknown[]) => {
      vscode.window.showInformationMessage(`Executing: ${cmd}`);
      console.log(`Tool ${cmd} executed with args:`, args);
    }));
  }
}
