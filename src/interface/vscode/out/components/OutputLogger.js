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
exports.OutputLogger = exports.LogLevel = void 0;
const vscode = __importStar(require("vscode"));
var LogLevel;
(function (LogLevel) {
    LogLevel[LogLevel["DEBUG"] = 0] = "DEBUG";
    LogLevel[LogLevel["INFO"] = 1] = "INFO";
    LogLevel[LogLevel["WARN"] = 2] = "WARN";
    LogLevel[LogLevel["ERROR"] = 3] = "ERROR";
})(LogLevel || (exports.LogLevel = LogLevel = {}));
const LOG_CONFIG = {
    [LogLevel.DEBUG]: { color: "#6C7086", label: "DEBUG" },
    [LogLevel.INFO]: { color: "#89B4FA", label: "INFO" },
    [LogLevel.WARN]: { color: "#F9E2AF", label: "WARN" },
    [LogLevel.ERROR]: { color: "#F38BA8", label: "ERROR" },
};
class OutputLogger {
    constructor() {
        this.minLevel = LogLevel.DEBUG;
        this.logs = [];
        this.channel = vscode.window.createOutputChannel("Hajimi");
    }
    static getInstance() {
        if (!OutputLogger.instance) {
            OutputLogger.instance = new OutputLogger();
        }
        return OutputLogger.instance;
    }
    setMinLevel(level) {
        this.minLevel = level;
    }
    formatTime() {
        const now = new Date();
        return now.toLocaleTimeString("en-US", { hour12: false }) + "." + String(now.getMilliseconds()).padStart(3, "0");
    }
    append(level, component, message) {
        if (level < this.minLevel)
            return;
        const cfg = LOG_CONFIG[level];
        const line = `[${cfg.label}] [${this.formatTime()}] [${component}] ${message}`;
        this.logs.push(line);
        this.channel.appendLine(line);
    }
    debug(component, message) {
        this.append(LogLevel.DEBUG, component, message);
    }
    info(component, message) {
        this.append(LogLevel.INFO, component, message);
    }
    warn(component, message) {
        this.append(LogLevel.WARN, component, message);
    }
    error(component, message) {
        this.append(LogLevel.ERROR, component, message);
    }
    show() {
        this.channel.show();
    }
    clear() {
        this.logs = [];
        this.channel.clear();
    }
    export() {
        return this.logs.join("\n");
    }
    filter(level) {
        return this.logs.filter((line) => line.includes(`[${LOG_CONFIG[level].label}]`));
    }
    dispose() {
        this.channel.dispose();
        OutputLogger.instance = undefined;
    }
}
exports.OutputLogger = OutputLogger;
//# sourceMappingURL=OutputLogger.js.map