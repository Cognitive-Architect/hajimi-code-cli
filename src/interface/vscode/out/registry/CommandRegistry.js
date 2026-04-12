"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.CommandRegistry = exports.CommandId = void 0;
const vscode = __importStar(require("vscode"));
// COMMAND: 56 tools + 4 shortcuts = 60 commands
var CommandId;
(function (CommandId) {
    CommandId["OPEN_SIDEBAR"] = "hajimi.openSidebar";
    CommandId["SEARCH_CODE"] = "hajimi.searchCode";
    CommandId["QUICK_COMMAND"] = "hajimi.quickCommand";
    CommandId["TOGGLE_TERMINAL"] = "hajimi.toggleTerminal";
    CommandId["EVM_COMPILE"] = "hajimi.evm.compile";
    CommandId["EVM_DEPLOY"] = "hajimi.evm.deploy";
    CommandId["EVM_VERIFY"] = "hajimi.evm.verify";
    CommandId["EVM_TEST"] = "hajimi.evm.test";
    CommandId["EVM_DEBUG"] = "hajimi.evm.debug";
    CommandId["EVM_ANALYZE"] = "hajimi.evm.analyze";
    CommandId["EVM_PATCH"] = "hajimi.evm.patch";
    CommandId["EVM_EXPLOIT"] = "hajimi.evm.exploit";
    CommandId["MCP_START"] = "hajimi.mcp.start";
    CommandId["MCP_STOP"] = "hajimi.mcp.stop";
    CommandId["MCP_RESTART"] = "hajimi.mcp.restart";
    CommandId["MCP_STATUS"] = "hajimi.mcp.status";
    CommandId["MCP_CONNECT"] = "hajimi.mcp.connect";
    CommandId["MCP_DISCONNECT"] = "hajimi.mcp.disconnect";
    CommandId["P2P_INIT"] = "hajimi.p2p.init";
    CommandId["P2P_SYNC"] = "hajimi.p2p.sync";
    CommandId["P2P_SHARE"] = "hajimi.p2p.share";
    CommandId["P2P_JOIN"] = "hajimi.p2p.join";
    CommandId["P2P_LEAVE"] = "hajimi.p2p.leave";
    CommandId["DB_CONNECT"] = "hajimi.db.connect";
    CommandId["DB_QUERY"] = "hajimi.db.query";
    CommandId["DB_MIGRATE"] = "hajimi.db.migrate";
    CommandId["DB_BACKUP"] = "hajimi.db.backup";
    CommandId["DB_RESTORE"] = "hajimi.db.restore";
    CommandId["WASM_BUILD"] = "hajimi.wasm.build";
    CommandId["WASM_RUN"] = "hajimi.wasm.run";
    CommandId["WASM_TEST"] = "hajimi.wasm.test";
    CommandId["FORMAT"] = "hajimi.format";
    CommandId["LINT"] = "hajimi.lint";
    CommandId["BUILD"] = "hajimi.build";
    CommandId["CLEAN"] = "hajimi.clean";
    CommandId["INSTALL"] = "hajimi.install";
    CommandId["UPDATE"] = "hajimi.update";
    CommandId["PUBLISH"] = "hajimi.publish";
    CommandId["PACKAGE"] = "hajimi.package";
    CommandId["DOCKER_BUILD"] = "hajimi.docker.build";
    CommandId["DOCKER_RUN"] = "hajimi.docker.run";
    CommandId["DOCKER_STOP"] = "hajimi.docker.stop";
    CommandId["GIT_COMMIT"] = "hajimi.git.commit";
    CommandId["GIT_PUSH"] = "hajimi.git.push";
    CommandId["GIT_PULL"] = "hajimi.git.pull";
    CommandId["GIT_BRANCH"] = "hajimi.git.branch";
    CommandId["GIT_MERGE"] = "hajimi.git.merge";
    CommandId["GIT_REBASE"] = "hajimi.git.rebase";
    CommandId["TEST_RUN"] = "hajimi.test.run";
    CommandId["TEST_DEBUG"] = "hajimi.test.debug";
    CommandId["TEST_COVERAGE"] = "hajimi.test.coverage";
    CommandId["BENCHMARK"] = "hajimi.benchmark";
    CommandId["PROFILE"] = "hajimi.profile";
    CommandId["AUDIT"] = "hajimi.audit";
    CommandId["SCAN"] = "hajimi.scan";
    CommandId["GENERATE"] = "hajimi.generate";
    CommandId["TEMPLATE"] = "hajimi.template";
    CommandId["CONFIGURE"] = "hajimi.configure";
    CommandId["VALIDATE"] = "hajimi.validate";
    CommandId["EXPORT"] = "hajimi.export";
})(CommandId || (exports.CommandId = CommandId = {}));
class CommandRegistry {
    constructor(context) {
        this.context = context;
    }
    registerCommand(command, callback) {
        this.context.subscriptions.push(vscode.commands.registerCommand(command, callback));
    }
    registerAllCommands() {
        this.registerCommand(CommandId.OPEN_SIDEBAR, () => vscode.commands.executeCommand('workbench.view.extension.hajimi'));
        this.registerCommand(CommandId.SEARCH_CODE, () => vscode.commands.executeCommand('workbench.action.findInFiles'));
        this.registerCommand(CommandId.QUICK_COMMAND, () => vscode.commands.executeCommand('workbench.action.showCommands'));
        this.registerCommand(CommandId.TOGGLE_TERMINAL, () => vscode.commands.executeCommand('workbench.action.terminal.toggleTerminal'));
        Object.values(CommandId).slice(4).forEach(cmd => this.registerCommand(cmd, (...args) => {
            vscode.window.showInformationMessage(`Executing: ${cmd}`);
            console.log(`Tool ${cmd} executed with args:`, args);
        }));
    }
}
exports.CommandRegistry = CommandRegistry;
//# sourceMappingURL=CommandRegistry.js.map