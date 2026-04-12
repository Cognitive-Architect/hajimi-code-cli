import * as vscode from "vscode";
export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
}
interface LogConfig {
  color: string;
  label: string;
}
const LOG_CONFIG: Record<LogLevel, LogConfig> = {
  [LogLevel.DEBUG]: { color: "#6C7086", label: "DEBUG" },
  [LogLevel.INFO]: { color: "#89B4FA", label: "INFO" },
  [LogLevel.WARN]: { color: "#F9E2AF", label: "WARN" },
  [LogLevel.ERROR]: { color: "#F38BA8", label: "ERROR" },
};
export class OutputLogger {
  private static instance: OutputLogger | undefined;
  private channel: vscode.OutputChannel;
  private minLevel: LogLevel = LogLevel.DEBUG;
  private logs: string[] = [];
  private constructor() {
    this.channel = vscode.window.createOutputChannel("Hajimi");
  }
  static getInstance(): OutputLogger {
    if (!OutputLogger.instance) {
      OutputLogger.instance = new OutputLogger();
    }
    return OutputLogger.instance;
  }
  setMinLevel(level: LogLevel): void {
    this.minLevel = level;
  }
  private formatTime(): string {
    const now = new Date();
    return now.toLocaleTimeString("en-US", { hour12: false }) + "." + String(now.getMilliseconds()).padStart(3, "0");
  }
  private append(level: LogLevel, component: string, message: string): void {
    if (level < this.minLevel) return;
    const cfg = LOG_CONFIG[level];
    const line = `[${cfg.label}] [${this.formatTime()}] [${component}] ${message}`;
    this.logs.push(line);
    this.channel.appendLine(line);
  }
  debug(component: string, message: string): void {
    this.append(LogLevel.DEBUG, component, message);
  }
  info(component: string, message: string): void {
    this.append(LogLevel.INFO, component, message);
  }
  warn(component: string, message: string): void {
    this.append(LogLevel.WARN, component, message);
  }
  error(component: string, message: string): void {
    this.append(LogLevel.ERROR, component, message);
  }
  show(): void {
    this.channel.show();
  }
  clear(): void {
    this.logs = [];
    this.channel.clear();
  }
  export(): string {
    return this.logs.join("\n");
  }
  filter(level: LogLevel): string[] {
    return this.logs.filter((line) => line.includes(`[${LOG_CONFIG[level].label}]`));
  }
  dispose(): void {
    this.channel.dispose();
    OutputLogger.instance = undefined;
  }
}
